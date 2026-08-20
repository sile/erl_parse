//! Sans I/O parser core.
//!
//! The parser owns a [`SyntaxTree`] (which bundles the growing
//! `TokenBuffer`, the append-only `SyntaxIndex`, and the accumulated
//! `ParseError`s) plus an internal event log that grammar functions
//! extend as they run. It exposes a small feed/pull API:
//!
//! - [`Parser::push_token`] appends one input token.
//! - [`Parser::next_top_node`] pulls the next completed top-level unit.
//! - [`Parser::state`] queries in-progress grammar state.
//! - [`Parser::syntax_tree`] borrows the accumulated tree for reading.
//! - [`Parser::finish`] consumes the parser and hands back the finished
//!   [`SyntaxTree`].
//!
//! Grammar functions (added in subsequent changes) consume tokens through
//! an internal cursor and emit start/finish events via [`Marker`] and
//! [`CompletedMarker`]; the parser drains completed top-level units into
//! the syntax index at boundaries.

use erl_tokenize::Token;

use crate::cursor::{CursorCheckpoint, TokenCursor};
use crate::error::{Expected, ParseError, ParseErrorKind, ProtocolError};
use crate::event::Event;
use crate::grammar::expr::parse_expr;
use crate::grammar::guard::parse_guard;
use crate::grammar::pattern::parse_pattern;
use crate::grammar::term::parse_term;
use crate::syntax::{EntryIndex, NodeId, SyntaxEntry, SyntaxIndex, SyntaxKind};
use crate::syntax_tree::SyntaxTree;
use crate::token_range::{TokenIndex, TokenRange};

/// Selects the top-level construct the parser recognizes.
///
/// This is fixed at construction time. The concrete grammar for each mode
/// is filled in by later changes; the stub grammar currently used here
/// treats a `.` token as the top-level boundary in every mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseMode {
    /// A `.erl` module: a sequence of forms terminated by `.`.
    Module,
    /// A `file:consult/1`-style file: a sequence of terms terminated by `.`.
    TermList,
    /// A single expression (an `erl_eval`-style input).
    Expression,
}

/// In-progress grammar state that a caller can query mid-unit.
///
/// This is a skeleton type; fields are filled in as grammar coverage grows.
/// Callers use it to obtain information such as the current form kind, the
/// attribute name, or the current function name / arity while a top-level
/// unit is being parsed, but this crate keeps only the shell.
#[derive(Debug, Default, Clone, Copy)]
pub struct InProgressState {
    _priv: (),
}

/// Which grammar sub-language the parser is currently accepting.
///
/// Set by [`Parser::set_context`] before calling grammar entry points
/// that layer position-specific restrictions on top of the shared
/// expression grammar. Grammar code queries this via
/// [`Parser::context`] and pushes a [`ParseError`] (without stalling
/// the cursor) when a construct is not allowed in the active context.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ParseContext {
    /// General expression position — every production is permitted.
    #[default]
    Expression,
    /// Left-hand-side pattern position — calls, blocks, funs,
    /// comprehensions, remote qualifiers, sends, and maybe-matches
    /// are rejected while the rest of the expression grammar is
    /// accepted structurally.
    Pattern,
    /// `file:consult/1`-style term position — the strictest subset:
    /// no variables, no matches, and none of the constructs already
    /// rejected in [`Pattern`][Self::Pattern].
    Term,
}

/// Sans I/O parser core.
#[derive(Debug)]
pub struct Parser {
    mode: ParseMode,
    tree: SyntaxTree,
    events: Vec<Event>,
    /// Full-token index the grammar has consumed up to.
    at: usize,
    /// Nesting depth counter maintained by grammar loops.
    depth: usize,
    /// Whether the stub grammar is currently inside a top-level unit.
    unit_in_progress: bool,
    /// If `unit_in_progress` is true, the event index of the current
    /// unit's outer `Start`.
    unit_start_event: Option<u32>,
    /// Where in `events` the last finalized unit ended (start of the next
    /// unit's events).
    unit_events_cursor: usize,
    /// Completed `NodeId`s produced by finalize but not yet handed out via
    /// `next_top_node`.
    pending_pull: std::collections::VecDeque<NodeId>,
    /// Skeleton in-progress state.
    in_progress: InProgressState,
    /// Which grammar sub-language is currently accepted (see
    /// [`ParseContext`]).
    context: ParseContext,
}

impl Parser {
    /// Creates a new parser for the given mode.
    pub fn new(mode: ParseMode) -> Self {
        Self {
            mode,
            tree: SyntaxTree::new(),
            events: Vec::new(),
            at: 0,
            depth: 0,
            unit_in_progress: false,
            unit_start_event: None,
            unit_events_cursor: 0,
            pending_pull: std::collections::VecDeque::new(),
            in_progress: InProgressState::default(),
            context: ParseContext::Expression,
        }
    }

    /// Returns the mode this parser was constructed with.
    pub fn mode(&self) -> ParseMode {
        self.mode
    }

    /// Appends a token to the internal buffer.
    pub fn push_token(&mut self, token: Token) {
        self.tree.tokens_mut().push(token);
        self.advance_grammar();
    }

    /// Returns the next completed top-level unit's [`NodeId`], or `None`
    /// when no new unit has completed since the last call.
    pub fn next_top_node(&mut self) -> Option<NodeId> {
        // Attempt to make grammar progress in case the previous push_token
        // paused at a partial unit.
        self.advance_grammar();
        self.pending_pull.pop_front()
    }

    /// Returns a snapshot of the in-progress grammar state.
    pub fn state(&self) -> InProgressState {
        self.in_progress
    }

    /// Borrows the accumulated syntax tree (tokens, syntax index, and
    /// errors) for reading during parsing.
    pub fn syntax_tree(&self) -> &SyntaxTree {
        &self.tree
    }

    /// Returns the current grammar nesting depth.
    pub fn nesting_depth(&self) -> usize {
        self.depth
    }

    /// Re-parses the tokens in `range` (which must already be inside the
    /// internal buffer) as a single expression. Returns the [`NodeId`]
    /// of the resulting top-level unit, which is appended to the syntax
    /// index alongside any units produced by the mode's top-level
    /// grammar.
    ///
    /// Fails with [`ProtocolError::AuxEntryPointWithUnitInProgress`]
    /// when a top-level unit is still open — callers must drain
    /// completed units via `next_top_node` (or start from a fresh
    /// parser) before invoking an auxiliary entry point.
    ///
    /// The cursor is saved before the sub-parse and restored after so
    /// the mode's top-level grammar continues from where it left off.
    pub fn parse_expression_range(&mut self, range: TokenRange) -> Result<NodeId, ProtocolError> {
        self.aux_parse(range, parse_expr)
    }

    /// Same shape as [`parse_expression_range`][Self::parse_expression_range]
    /// but parses the range as a single pattern.
    pub fn parse_pattern_range(&mut self, range: TokenRange) -> Result<NodeId, ProtocolError> {
        self.aux_parse(range, parse_pattern)
    }

    /// Same shape as [`parse_expression_range`][Self::parse_expression_range]
    /// but parses the range as a single guard sequence.
    pub fn parse_guard_range(&mut self, range: TokenRange) -> Result<NodeId, ProtocolError> {
        self.aux_parse(range, parse_guard)
    }

    /// Same shape as [`parse_expression_range`][Self::parse_expression_range]
    /// but parses the range as a single Erlang term.
    pub fn parse_term_range(&mut self, range: TokenRange) -> Result<NodeId, ProtocolError> {
        self.aux_parse(range, parse_term)
    }

    fn aux_parse<F>(&mut self, range: TokenRange, body: F) -> Result<NodeId, ProtocolError>
    where
        F: FnOnce(&mut Parser) -> crate::parser::CompletedMarker,
    {
        if self.unit_in_progress {
            return Err(ProtocolError::AuxEntryPointWithUnitInProgress);
        }
        let saved_at = self.at;
        self.at = range.start().get();
        // Sub-parse events belong to a fresh unit.
        self.unit_events_cursor = self.events.len();
        let _completed = body(self);
        self.finalize_pending_units();
        self.at = saved_at;
        Ok(self
            .pending_pull
            .pop_back()
            .expect("finalize_pending_units appends exactly one unit"))
    }

    /// Asserts end of input, consumes the parser, and returns the finished
    /// [`SyntaxTree`].
    ///
    /// If a top-level unit is still in progress an
    /// [`UnexpectedEof`][ParseErrorKind::UnexpectedEof] is appended to
    /// the error list and the unit is force-closed as
    /// [`SyntaxKind::Error`] so it survives in the returned tree.
    pub fn finish(mut self) -> SyntaxTree {
        self.advance_grammar();
        // Expression mode: if lexical tokens remain past the cursor
        // (the buffer ended without a terminating `.`), treat the
        // remaining input as the final expression unit.
        if self.mode == ParseMode::Expression && self.peek_lexical(0).is_some() {
            self.unit_events_cursor = self.events.len();
            let _completed = parse_expr(&mut self);
            self.finalize_pending_units();
        }
        if self.unit_in_progress {
            let end = self.tree.tokens().end_index();
            self.tree.errors_mut().push(ParseError::new(
                ParseErrorKind::UnexpectedEof,
                TokenRange::empty_at(end),
                Expected::Unspecified,
                None,
            ));
            // Force-complete the current unit as an Error node so callers
            // can still see it structurally.
            let event_index = self
                .unit_start_event
                .take()
                .expect("unit_in_progress implies unit_start_event is set");
            self.events.push(Event::Finish { end_at: end });
            self.set_marker_kind(event_index, SyntaxKind::Error);
            self.unit_in_progress = false;
            self.finalize_pending_units();
        }
        self.tree
    }

    // ---------------------------------------------------------------------
    // Internal API used by grammar code (and by the in-crate stub grammar).
    // ---------------------------------------------------------------------

    /// Starts a new tentative node at the current cursor position and
    /// returns a [`Marker`] handle.
    pub(crate) fn start(&mut self) -> Marker {
        let event_index = self.events.len() as u32;
        let start_at = TokenIndex::new(self.at);
        self.events.push(Event::Start {
            kind: None,
            forward_parent: None,
            start_at,
        });
        Marker {
            event_index,
            finished: false,
        }
    }

    /// Consumes one lexical token from the buffer, folding any trailing
    /// hidden tokens into the consumed span. Returns the boundary the
    /// cursor advanced to, or `None` when no lexical token is available.
    pub(crate) fn consume_lexical(&mut self) -> Option<TokenIndex> {
        let mut cursor = TokenCursor::new(self.tree.tokens(), self.at);
        let end = cursor.advance_lexical()?;
        self.at = end.get();
        Some(end)
    }

    /// Peeks the nth lexical token from the cursor position (0-based),
    /// skipping hidden tokens. Returns `None` when the requested lookahead
    /// is beyond the tokens that have been pushed so far.
    pub(crate) fn peek_lexical(&self, offset: usize) -> Option<(TokenIndex, Token)> {
        TokenCursor::new(self.tree.tokens(), self.at).peek_lexical(offset)
    }

    /// Returns the cursor's current position as a [`TokenIndex`], for use
    /// in [`ParseError`] ranges.
    pub(crate) fn cursor_position(&self) -> TokenIndex {
        TokenIndex::new(self.at)
    }

    /// Appends a [`ParseError`] to the accumulated errors.
    pub(crate) fn push_error(&mut self, error: ParseError) {
        self.tree.errors_mut().push(error);
    }

    /// Returns the current grammar sub-language ([`ParseContext`]).
    pub(crate) fn context(&self) -> ParseContext {
        self.context
    }

    /// Sets the current grammar sub-language and returns the previous
    /// value so callers can restore it. Grammar entry points for
    /// pattern / term / expression positions use this to switch the
    /// active restriction set for the span they own.
    pub(crate) fn set_context(&mut self, context: ParseContext) -> ParseContext {
        std::mem::replace(&mut self.context, context)
    }

    /// Resets stub-grammar state so a grammar module's tests can drive
    /// the cursor manually from position 0. Never called in production
    /// paths.
    #[cfg(test)]
    pub(crate) fn reset_for_test(&mut self) {
        self.at = 0;
        self.unit_in_progress = false;
        self.unit_start_event = None;
        self.unit_events_cursor = self.events.len();
        self.pending_pull.clear();
    }

    /// Drains completed top-level units into the syntax index, exposed
    /// for grammar-module tests that construct units manually.
    #[cfg(test)]
    pub(crate) fn finalize_pending_units_for_test(&mut self) {
        self.finalize_pending_units();
    }

    /// Returns `true` when the cursor has reached the end of the currently
    /// pushed buffer. Combine with [`Parser::is_finished`] to distinguish
    /// "waiting for more input" from "true EOF".
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Grammar code added later drives this API; only tests currently exercise it"
        )
    )]
    pub(crate) fn is_cursor_at_buffer_end(&self) -> bool {
        self.at >= self.tree.tokens().len()
    }

    /// Captures a checkpoint over `(cursor, event_log_len, error_sink_len)`
    /// so a failed alternative can be unwound without cloning any tokens.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Grammar code added later drives this API; only tests currently exercise it"
        )
    )]
    pub(crate) fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            cursor: TokenCursor::new(self.tree.tokens(), self.at).save(),
            events_len: self.events.len(),
            errors_len: self.tree.errors().len(),
        }
    }

    /// Restores a previously captured checkpoint. Any events, errors, or
    /// cursor advances made after the checkpoint are dropped.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Grammar code added later drives this API; only tests currently exercise it"
        )
    )]
    pub(crate) fn restore(&mut self, checkpoint: Checkpoint) {
        self.at = checkpoint.cursor.at();
        self.events.truncate(checkpoint.events_len);
        self.tree.errors_mut().truncate(checkpoint.errors_len);
    }

    /// Runs `body` inside a nesting-depth-tracked block. `body` returns the
    /// number of tokens it consumed; the helper panics if that count is
    /// zero, forcing every parser loop iteration to make forward progress.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Grammar code added later uses this helper; only tests currently exercise it"
        )
    )]
    pub(crate) fn advance_with<F>(&mut self, body: F)
    where
        F: FnOnce(&mut Parser) -> ProgressGuard,
    {
        let start_at = self.at;
        self.depth += 1;
        let _guard = body(self);
        self.depth -= 1;
        assert!(
            self.at > start_at,
            "grammar step must consume at least one token before returning"
        );
    }

    // ---------------------------------------------------------------------
    // Marker helpers (used by Marker / CompletedMarker methods below).
    // ---------------------------------------------------------------------

    fn set_marker_kind(&mut self, event_index: u32, kind: SyntaxKind) {
        match &mut self.events[event_index as usize] {
            Event::Start { kind: k, .. } => *k = Some(kind),
            _ => unreachable!("marker must point at a Start event"),
        }
    }

    fn set_forward_parent(&mut self, event_index: u32, parent_index: u32) {
        match &mut self.events[event_index as usize] {
            Event::Start { forward_parent, .. } => {
                *forward_parent = Some(parent_index - event_index);
            }
            _ => unreachable!("marker must point at a Start event"),
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Reachable only via Marker::abandon, which grammar code drives later; currently only tests exercise it"
        )
    )]
    fn convert_to_tombstone(&mut self, event_index: u32) {
        self.events[event_index as usize] = Event::Tombstone;
    }

    // ---------------------------------------------------------------------
    // Stub grammar and finalize.
    // ---------------------------------------------------------------------

    fn advance_grammar(&mut self) {
        match self.mode {
            ParseMode::Expression => self.advance_expression_grammar(),
            ParseMode::Module | ParseMode::TermList => self.advance_stub_grammar(),
        }
    }

    /// Expression-mode top-level driver: whenever a lexical `.` appears
    /// in the pending buffer, parse the tokens up to (and including) it
    /// as a single expression and emit a top-level unit. Tokens still
    /// unterminated by a `.` at push time stay in the buffer for the
    /// next call.
    fn advance_expression_grammar(&mut self) {
        while self.has_lexical_dot_after_cursor() {
            self.unit_events_cursor = self.events.len();
            let _completed = parse_expr(self);
            // parse_expr may have stopped before the terminating `.`
            // (for example on an unexpected token). Consume any leftover
            // lexical tokens up to and including the boundary dot; each
            // extra token is flagged as an unexpected-token error so
            // the diagnostic survives without stalling the cursor.
            while let Some((_, token)) = self.peek_lexical(0) {
                if is_dot(token) {
                    self.consume_lexical();
                    break;
                }
                self.push_error(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    TokenRange::empty_at(TokenIndex::new(self.at)),
                    Expected::Category("`.` to close top-level expression"),
                    Some(token),
                ));
                self.consume_lexical();
            }
            self.finalize_pending_units();
        }
    }

    /// Stub grammar: consume lexical tokens until a `.` is seen; then
    /// complete the current top-level unit as a `SyntaxKind::Error`
    /// node. Kept for module / term-list modes until a later change
    /// wires the corresponding real grammar in.
    fn advance_stub_grammar(&mut self) {
        loop {
            if !self.unit_in_progress {
                if self.peek_lexical(0).is_none() {
                    return;
                }
                let event_index = self.events.len() as u32;
                let start_at = TokenIndex::new(self.at);
                self.events.push(Event::Start {
                    kind: None,
                    forward_parent: None,
                    start_at,
                });
                self.unit_in_progress = true;
                self.unit_start_event = Some(event_index);
                let _ = start_at;
            }

            loop {
                let Some((_idx, token)) = self.peek_lexical(0) else {
                    return;
                };
                self.consume_lexical()
                    .expect("peek_lexical returned Some so a lexical token exists");
                if is_dot(token) {
                    let event_index = self
                        .unit_start_event
                        .take()
                        .expect("unit_in_progress implies unit_start_event is set");
                    let end_at = TokenIndex::new(self.at);
                    self.set_marker_kind(event_index, SyntaxKind::Error);
                    self.events.push(Event::Finish { end_at });
                    self.unit_in_progress = false;
                    self.finalize_pending_units();
                    break;
                }
            }
        }
    }

    /// Scans the pending buffer for a lexical `.` at or after the
    /// current cursor position, without moving the cursor.
    fn has_lexical_dot_after_cursor(&self) -> bool {
        let tokens = self.tree.tokens();
        let mut i = self.at;
        while let Some(t) = tokens.get(TokenIndex::new(i)) {
            if t.kind().is_lexical() && is_dot(t) {
                return true;
            }
            i += 1;
        }
        false
    }

    fn finalize_pending_units(&mut self) {
        let events_start = self.unit_events_cursor;
        let events_end = self.events.len();
        self.unit_events_cursor = events_end;

        let root_id = finalize_unit(&self.events[events_start..], self.tree.syntax_mut());
        if let Some(id) = root_id {
            self.pending_pull.push_back(id);
        }
    }
}

fn is_dot(token: Token) -> bool {
    matches!(
        token.kind(),
        erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::Dot)
    )
}

/// Handle to an open node in the event log.
///
/// The grammar creates one by calling [`Parser::start`]. It must be either
/// [`Marker::complete`]d with a concrete [`SyntaxKind`] or
/// [`Marker::abandon`]ed. Dropping an unfinished marker panics on Drop so
/// that leaving markers open is treated as an implementation bug.
#[derive(Debug)]
pub(crate) struct Marker {
    event_index: u32,
    finished: bool,
}

impl Marker {
    /// Completes this marker with the given [`SyntaxKind`], sealing the
    /// current node in the event log and returning a [`CompletedMarker`]
    /// that can still be preceded by a wrapping parent.
    pub(crate) fn complete(mut self, parser: &mut Parser, kind: SyntaxKind) -> CompletedMarker {
        parser.set_marker_kind(self.event_index, kind);
        let end_at = TokenIndex::new(parser.at);
        parser.events.push(Event::Finish { end_at });
        self.finished = true;
        CompletedMarker {
            event_index: self.event_index,
        }
    }

    /// Abandons this marker, converting the reserved `Start` event into a
    /// tombstone so it is skipped during finalize.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Grammar code added later drives this API; only tests currently exercise it"
        )
    )]
    pub(crate) fn abandon(mut self, parser: &mut Parser) {
        parser.convert_to_tombstone(self.event_index);
        self.finished = true;
        // Note: the token cursor is not rewound; abandoning is meant for
        // markers that only reserved the Start slot without consuming
        // tokens. Callers who need to unwind consumed tokens should use
        // `Parser::checkpoint` / `Parser::restore` instead.
    }
}

impl Drop for Marker {
    fn drop(&mut self) {
        if !self.finished && !std::thread::panicking() {
            panic!("Marker dropped without being completed or abandoned");
        }
    }
}

/// Handle to a node that has already been completed. Can be used to
/// retroactively wrap it in a parent node via [`CompletedMarker::precede`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct CompletedMarker {
    event_index: u32,
}

impl CompletedMarker {
    /// Creates a new marker that will become the parent of this completed
    /// node once the returned [`Marker`] is completed. Used for
    /// Pratt-style promotion: after parsing a left-hand expression, the
    /// operator can retroactively make the left-hand node its child.
    pub(crate) fn precede(self, parser: &mut Parser) -> Marker {
        let new_event_index = parser.events.len() as u32;
        parser.set_forward_parent(self.event_index, new_event_index);
        // The wrapping parent inherits the wrapped node's start position;
        // Pratt promotion never consumes new tokens.
        let start_at = match parser.events[self.event_index as usize] {
            Event::Start { start_at, .. } => start_at,
            _ => unreachable!("CompletedMarker must point at a Start event"),
        };
        parser.events.push(Event::Start {
            kind: None,
            forward_parent: None,
            start_at,
        });
        Marker {
            event_index: new_event_index,
            finished: false,
        }
    }
}

/// Bundle of `(cursor, events_len, errors_len)` used by grammar to unwind
/// after a failed alternative.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Checkpoint {
    cursor: CursorCheckpoint,
    events_len: usize,
    errors_len: usize,
}

/// Zero-sized proof returned by parser-loop bodies that at least one token
/// was consumed during the step; enforced by [`Parser::advance_with`].
#[derive(Debug)]
pub(crate) struct ProgressGuard {
    _priv: (),
}

impl ProgressGuard {
    /// Constructs a `ProgressGuard`. Callers should only construct this
    /// after actually consuming at least one token.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Grammar code added later constructs this after consuming; only tests currently exercise it"
        )
    )]
    pub(crate) const fn new() -> Self {
        Self { _priv: () }
    }
}

// -------------------------------------------------------------------------
// Finalize: turn a slice of events into SyntaxIndex entries.
// -------------------------------------------------------------------------

fn finalize_unit(events: &[Event], index: &mut SyntaxIndex) -> Option<NodeId> {
    // Rowan-style finalize:
    // - Walk events in order.
    // - When we see a `Start` with a `forward_parent`, we must emit that
    //   parent's Start first (outer-first). Follow the chain to the
    //   outermost ancestor and emit them in outer-to-inner order. Mark
    //   visited ones so a later direct visit skips them.
    // - `Finish` closes the innermost open Start on the stack; we backfill
    //   its range and subtree_end.
    // - `Tombstone`s are skipped.
    //
    // Entries are built into a local buffer first so we can backfill in
    // place, and then bulk-appended to the append-only `SyntaxIndex`.

    let mut visited = vec![false; events.len()];
    let mut local: Vec<SyntaxEntry> = Vec::new();
    let mut open_stack: Vec<(usize, TokenIndex, SyntaxKind)> = Vec::new();

    for i in 0..events.len() {
        if visited[i] {
            continue;
        }
        match events[i] {
            Event::Start {
                kind,
                forward_parent,
                start_at,
            } => {
                visited[i] = true;

                // Follow forward_parent chain. `chain` holds ancestors in
                // inner-first order (index i first). Each ancestor
                // contributes its own kind but shares the innermost node's
                // start position (Pratt promotion does not consume tokens).
                let mut chain: Vec<SyntaxKind> = Vec::new();
                chain.push(
                    kind.expect("finalize: Marker must be completed with a kind before finalize"),
                );
                let mut cur = i;
                let mut cur_delta = forward_parent;
                while let Some(delta) = cur_delta {
                    let ancestor_i = cur + delta as usize;
                    assert!(
                        !visited[ancestor_i],
                        "forward_parent must point at an unvisited Start"
                    );
                    visited[ancestor_i] = true;
                    match events[ancestor_i] {
                        Event::Start {
                            kind: k,
                            forward_parent: fp,
                            start_at: _,
                        } => {
                            chain.push(k.expect(
                                "finalize: forward-parent Marker must be completed with a kind",
                            ));
                            cur = ancestor_i;
                            cur_delta = fp;
                        }
                        _ => unreachable!("forward_parent must point at a Start"),
                    }
                }

                // Emit outermost-first.
                for &kind in chain.iter().rev() {
                    let position = local.len();
                    local.push(SyntaxEntry::new(
                        kind,
                        TokenRange::empty_at(start_at),
                        EntryIndex::new(position + 1),
                    ));
                    open_stack.push((position, start_at, kind));
                }
            }
            Event::Finish { end_at } => {
                let (position, start_at, kind) = open_stack
                    .pop()
                    .expect("finalize: Finish without a matching open Start (unbalanced events)");
                let subtree_end = EntryIndex::new(local.len());
                let range = TokenRange::new(start_at, end_at);
                assert!(
                    subtree_end.get() > position,
                    "finalize: subtree_end must satisfy self_index + 1 <= subtree_end"
                );
                local[position] = SyntaxEntry::new(kind, range, subtree_end);
                visited[i] = true;
            }
            Event::Tombstone => {
                visited[i] = true;
            }
        }
    }

    assert!(
        open_stack.is_empty(),
        "finalize: {} unclosed open marker(s) remain",
        open_stack.len()
    );

    if local.is_empty() {
        return None;
    }
    let root_id = NodeId::new(index.len());
    for entry in local {
        index.push(entry);
    }
    Some(root_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use erl_tokenize::{Position, scan_token};

    fn scan_all(source: &str) -> Vec<Token> {
        let mut out = Vec::new();
        let mut pos = Position::new();
        while let Some(t) = scan_token(source, pos).expect("valid source") {
            out.push(t);
            pos = t.end();
        }
        out
    }

    fn push_all(parser: &mut Parser, source: &str) {
        for t in scan_all(source) {
            parser.push_token(t);
        }
    }

    #[test]
    fn empty_buffer_pulls_nothing() {
        let mut p = Parser::new(ParseMode::Module);
        assert!(p.next_top_node().is_none());
        assert!(p.syntax_tree().syntax().is_empty());
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn hidden_only_buffer_yields_nothing_but_stores_tokens() {
        let mut p = Parser::new(ParseMode::Module);
        push_all(&mut p, "   \n  ");
        assert!(!p.syntax_tree().tokens().is_empty());
        assert!(p.next_top_node().is_none());
    }

    #[test]
    fn pull_returns_none_before_boundary_and_node_id_after() {
        let mut p = Parser::new(ParseMode::Module);
        push_all(&mut p, "foo");
        assert!(p.next_top_node().is_none(), "no dot yet");
        push_all(&mut p, " .");
        let node = p.next_top_node().expect("unit completed at dot");
        assert_eq!(node, NodeId::new(0));
        assert_eq!(p.syntax_tree().syntax().len(), 1);
        assert!(p.next_top_node().is_none(), "one unit only");
    }

    #[test]
    fn buffer_end_reports_true_before_any_push() {
        let p = Parser::new(ParseMode::Module);
        assert!(p.is_cursor_at_buffer_end());
    }

    #[test]
    fn finish_returns_syntax_tree_with_completed_units() {
        let mut p = Parser::new(ParseMode::Module);
        push_all(&mut p, "foo. bar.");
        let tree = p.finish();
        assert_eq!(tree.syntax().len(), 2);
        assert!(tree.errors().is_empty());
    }

    #[test]
    fn finish_flushes_incomplete_unit_as_error() {
        let mut p = Parser::new(ParseMode::Module);
        push_all(&mut p, "foo bar");
        assert!(p.next_top_node().is_none());
        let tree = p.finish();
        assert_eq!(tree.syntax().len(), 1);
        assert_eq!(tree.errors().len(), 1);
        assert_eq!(tree.errors()[0].kind(), ParseErrorKind::UnexpectedEof);
    }

    #[test]
    fn determinism_across_two_parsers() {
        let source = "foo. bar.";
        let mut a = Parser::new(ParseMode::Module);
        let mut b = Parser::new(ParseMode::Module);
        push_all(&mut a, source);
        push_all(&mut b, source);
        let ta = a.syntax_tree();
        let tb = b.syntax_tree();
        assert_eq!(ta.syntax().len(), tb.syntax().len());
        assert_eq!(ta.errors().len(), tb.errors().len());
        for i in 0..ta.syntax().len() {
            assert_eq!(
                ta.syntax().entry(NodeId::new(i)),
                tb.syntax().entry(NodeId::new(i)),
                "entry {} differs",
                i
            );
        }
    }

    #[test]
    fn hidden_tokens_between_lexical_tokens_are_folded_into_node_range() {
        let mut p = Parser::new(ParseMode::Module);
        // Interior whitespace between `foo` and `.` is folded into the
        // consumed span by advance_lexical, so the unit's range extends
        // through the closing dot.
        push_all(&mut p, "foo .");
        let node = p.next_top_node().expect("unit completed");
        let tree = p.syntax_tree();
        let entry = tree.syntax().entry(node).expect("entry exists");
        assert_eq!(entry.range().start(), TokenIndex::new(0));
        assert_eq!(entry.range().end(), tree.tokens().end_index());
    }

    #[test]
    fn trailing_hidden_tokens_pushed_after_boundary_are_not_folded_in() {
        let mut p = Parser::new(ParseMode::Module);
        // Whitespace pushed after the closing dot cannot be folded into a
        // unit that already completed; it stays in the buffer past the
        // unit's end.
        push_all(&mut p, "foo . ");
        let node = p.next_top_node().expect("unit completed");
        let tree = p.syntax_tree();
        let entry = tree.syntax().entry(node).expect("entry exists");
        assert!(entry.range().end() < tree.tokens().end_index());
    }

    #[test]
    fn checkpoint_restores_cursor_events_and_errors() {
        let mut p = Parser::new(ParseMode::Module);
        // Push two lexical tokens without a boundary so the stub grammar
        // consumes them into an in-progress unit; then reset the cursor
        // manually so this test can drive the checkpoint API directly.
        push_all(&mut p, "foo bar");
        p.at = 0;
        p.unit_in_progress = false;
        p.unit_events_cursor = p.events.len();
        p.tree.errors_mut().clear();

        let ckpt = p.checkpoint();
        let before_at = p.at;
        let before_events = p.events.len();
        // Advance the cursor and add a speculative error.
        let _ = p.consume_lexical();
        p.tree.errors_mut().push(ParseError::new(
            ParseErrorKind::UnexpectedToken,
            TokenRange::empty_at(TokenIndex::new(0)),
            Expected::Unspecified,
            None,
        ));
        assert!(p.at > before_at);
        assert_eq!(p.syntax_tree().errors().len(), 1);

        p.restore(ckpt);
        assert_eq!(p.at, before_at);
        assert_eq!(p.events.len(), before_events);
        assert!(
            p.syntax_tree().errors().is_empty(),
            "restored errors are dropped"
        );
    }

    #[test]
    fn marker_complete_produces_syntax_entry() {
        let mut p = Parser::new(ParseMode::Expression);
        push_all(&mut p, "foo");
        // Reset internal cursor to 0 so we can manually build a unit.
        // (The stub grammar has already consumed to the end.)
        p.at = 0;
        p.unit_in_progress = false;
        p.unit_events_cursor = p.events.len();
        p.pending_pull.clear();

        let outer = p.start();
        let _ = p.consume_lexical();
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units();
        let node = p.next_top_node().expect("outer unit");
        let entry = p.syntax_tree().syntax().entry(node).expect("entry exists");
        assert_eq!(entry.range().start(), TokenIndex::new(0));
    }

    #[test]
    fn completed_marker_precede_makes_pratt_wrapper() {
        let mut p = Parser::new(ParseMode::Expression);
        push_all(&mut p, "foo");
        // Reset for manual scenario.
        p.at = 0;
        p.unit_in_progress = false;
        p.unit_events_cursor = p.events.len();
        p.pending_pull.clear();

        let inner = p.start();
        let _ = p.consume_lexical();
        let completed = inner.complete(&mut p, SyntaxKind::Error);
        // Retroactively wrap the completed inner as a child of a new outer.
        let outer = completed.precede(&mut p);
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units();

        let root = p.next_top_node().expect("root");
        assert_eq!(root, NodeId::new(0));
        let index = p.syntax_tree().syntax();
        let outer_entry = index.entry(root).expect("outer entry exists");
        // The outer wraps the inner, so its subtree spans two entries.
        assert_eq!(outer_entry.subtree_end().get() - root.get(), 2);
        let inner_id = NodeId::new(1);
        let inner_entry = index.entry(inner_id).expect("inner entry exists");
        // Both share the same range (the inner + wrapper occupy the same
        // consumed tokens; the wrapper does not add new consumption).
        assert_eq!(outer_entry.range(), inner_entry.range());
    }

    #[test]
    fn marker_abandon_leaves_tombstone_that_finalize_skips() {
        let mut p = Parser::new(ParseMode::Expression);
        push_all(&mut p, "foo");
        p.at = 0;
        p.unit_in_progress = false;
        p.unit_events_cursor = p.events.len();
        p.pending_pull.clear();

        let outer = p.start();
        // Reserve an inner marker slot and then abandon it before doing
        // any work. The outer marker still becomes the top-level node.
        let inner = p.start();
        inner.abandon(&mut p);
        let _ = p.consume_lexical();
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units();

        let root = p.next_top_node().expect("outer unit");
        assert_eq!(root, NodeId::new(0));
        // Only one entry: the outer node. The abandoned inner produced no
        // syntax entry.
        assert_eq!(p.syntax_tree().syntax().len(), 1);
    }

    #[test]
    #[should_panic(expected = "Marker dropped without being completed")]
    fn dropping_unfinished_marker_panics() {
        let mut p = Parser::new(ParseMode::Module);
        let _m = p.start();
        // dropped here without complete or abandon
    }

    #[test]
    fn advance_with_forbids_zero_consumption() {
        // Positive case: consuming one token is fine.
        let mut p = Parser::new(ParseMode::Module);
        push_all(&mut p, "foo");
        p.at = 0;
        p.unit_in_progress = false;
        p.unit_events_cursor = p.events.len();
        p.pending_pull.clear();
        p.advance_with(|inner| {
            let _ = inner.consume_lexical();
            ProgressGuard::new()
        });
        assert!(p.at > 0);
    }

    #[test]
    #[should_panic(expected = "consume at least one token")]
    fn advance_with_panics_on_zero_consumption() {
        let mut p = Parser::new(ParseMode::Module);
        push_all(&mut p, "foo");
        p.at = 0;
        p.unit_in_progress = false;
        p.unit_events_cursor = p.events.len();
        p.pending_pull.clear();
        p.advance_with(|_inner| ProgressGuard::new());
    }
}
