//! Shared generators, coverage counters, and validation helpers for
//! the `pbt_*` property-based tests.
//!
//! Cargo compiles every `tests/*.rs` as its own integration-test
//! binary, so each binary only uses a subset of these helpers.
//! Silence `dead_code` on the `mod pbt_harness;` declaration in each
//! consuming file, not here: a crate-level `expect` is unfulfilled
//! when this file is itself a test target (its `pub` items are the
//! crate's public API).

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;

use erl_parse::{
    EntryIndex, NodeId, ParseError, ParseErrorKind, ParseMode, Parser, SyntaxKind, SyntaxTree,
    TokenBuffer, TokenIndex, TokenRange,
};
use erl_tokenize::{Position, Token, scan_token};

/// Environment variable read by `noprop::seed_from_env_or_time` to
/// reproduce a failing case: `ERL_PARSE_PBT_SEED=<seed>`.
pub const SEED_ENV: &str = "ERL_PARSE_PBT_SEED";

/// Default number of accepted cases each property runs.
pub const CASES: usize = 256;

/// Upper bound on generated source length in characters; keeps
/// generated cases small enough to explore quickly.
pub const MAX_SOURCE_LEN: usize = 256;

/// Upper bound on grammar recursion depth used by the generators.
/// Kept low so ordinary cases finish quickly (a naive depth of 6
/// with a branching factor near 3 explodes into thousands of
/// tokens per case); a dedicated deep-nesting generator exceeds
/// `Parser::MAX_NESTING_DEPTH` on purpose for the cap-check test.
pub const MAX_GEN_DEPTH: usize = 3;

/// Upper bound on comma-separated child count inside a compound
/// (tuple / list / argument list). Kept small so total node count
/// stays under a few dozen per case.
pub const MAX_CHILDREN: usize = 2;

// -----------------------------------------------------------------
// Coverage counters (interior mutability so the closure implements
// `Fn`). Use the smallest shape that captures the meaning of the
// gate: `Flag` for "was ever reached", `Counter` for a count, and
// `LabelSet` for a set of distinct labels seen across cases.
// -----------------------------------------------------------------

/// "Did the property ever hit this branch." `assert!(flag.hit(), ...)`
/// after the run.
#[derive(Debug, Default)]
pub struct Flag(Cell<bool>);

impl Flag {
    pub fn new() -> Self {
        Self(Cell::new(false))
    }
    pub fn set(&self) {
        self.0.set(true);
    }
    pub fn hit(&self) -> bool {
        self.0.get()
    }
}

/// "How many times did the property hit this branch."
#[derive(Debug, Default)]
pub struct Counter(Cell<usize>);

impl Counter {
    pub fn new() -> Self {
        Self(Cell::new(0))
    }
    pub fn bump(&self) {
        self.0.set(self.0.get() + 1);
    }
    pub fn get(&self) -> usize {
        self.0.get()
    }
}

/// Set of distinct labels observed across cases (each label added
/// once). Reads through `contains` after the run to gate coverage.
#[derive(Debug, Default)]
pub struct LabelSet(RefCell<BTreeSet<&'static str>>);

impl LabelSet {
    pub fn new() -> Self {
        Self(RefCell::new(BTreeSet::new()))
    }
    pub fn insert(&self, label: &'static str) {
        self.0.borrow_mut().insert(label);
    }
    pub fn contains(&self, label: &'static str) -> bool {
        self.0.borrow().contains(label)
    }
    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }
}

// -----------------------------------------------------------------
// Source generators. Every generator returns a String that the
// scanner accepts; grammar-invalid but scanner-valid strings are
// intentional (they exercise the recovery path).
// -----------------------------------------------------------------

/// Uniform atom name from a small alphabet. Kept short so tuples of
/// atoms fit within `MAX_SOURCE_LEN`.
pub fn sample_atom_name(ctx: &mut noprop::TestCaseContext) -> &'static str {
    noprop::sample_choice(ctx, &["a", "b", "c", "foo", "bar", "ok", "error"])
}

/// Uniform variable name from a small alphabet.
pub fn sample_var_name(ctx: &mut noprop::TestCaseContext) -> &'static str {
    noprop::sample_choice(ctx, &["X", "Y", "Z", "Acc", "Val", "_"])
}

/// Uniform small integer literal.
pub fn sample_integer_literal(ctx: &mut noprop::TestCaseContext) -> String {
    let v = noprop::sample_usize_in(ctx, 0..=99);
    v.to_string()
}

/// Draws a leaf expression source string (no recursion).
pub fn sample_leaf(ctx: &mut noprop::TestCaseContext) -> String {
    match noprop::sample_weighted_index(ctx, &[3, 3, 2, 1, 1]) {
        0 => sample_atom_name(ctx).to_string(),
        1 => sample_integer_literal(ctx),
        2 => sample_var_name(ctx).to_string(),
        3 => "\"s\"".to_string(),
        _ => "$a".to_string(),
    }
}

/// Draws an expression source string of bounded depth. `depth ==
/// 0` returns a leaf; otherwise picks a compound and recurses. The
/// picks include tuple, list, parenthesized, and binary op — enough
/// to exercise container / operator paths.
pub fn sample_expression(ctx: &mut noprop::TestCaseContext, depth: usize) -> String {
    if depth == 0 {
        return sample_leaf(ctx);
    }
    match noprop::sample_weighted_index(ctx, &[4, 3, 3, 2, 2, 1]) {
        0 => sample_leaf(ctx),
        1 => {
            let n = noprop::sample_usize_in(ctx, 0..=MAX_CHILDREN);
            let mut parts = Vec::with_capacity(n);
            for _ in 0..n {
                parts.push(sample_expression(ctx, depth - 1));
            }
            format!("{{{}}}", parts.join(", "))
        }
        2 => {
            let n = noprop::sample_usize_in(ctx, 0..=MAX_CHILDREN);
            let mut parts = Vec::with_capacity(n);
            for _ in 0..n {
                parts.push(sample_expression(ctx, depth - 1));
            }
            format!("[{}]", parts.join(", "))
        }
        3 => {
            let inner = sample_expression(ctx, depth - 1);
            format!("({inner})")
        }
        4 => {
            let lhs = sample_expression(ctx, depth - 1);
            let rhs = sample_expression(ctx, depth - 1);
            let op = noprop::sample_choice(ctx, &["+", "-", "*", "=="]);
            format!("{lhs} {op} {rhs}")
        }
        _ => {
            let inner = sample_expression(ctx, depth - 1);
            format!("- {inner}")
        }
    }
}

/// Wraps `sample_expression` with a terminating `.` so the result
/// is a valid expression-mode top-level unit.
pub fn sample_expression_unit(ctx: &mut noprop::TestCaseContext) -> String {
    let mut s = sample_expression(ctx, MAX_GEN_DEPTH);
    s.push('.');
    s
}

/// Draws a term source (subset of expression: no variables, no
/// match, no calls). Used to feed term-list mode.
pub fn sample_term(ctx: &mut noprop::TestCaseContext, depth: usize) -> String {
    if depth == 0 {
        return match noprop::sample_weighted_index(ctx, &[3, 3, 1, 1]) {
            0 => sample_atom_name(ctx).to_string(),
            1 => sample_integer_literal(ctx),
            2 => "\"t\"".to_string(),
            _ => "$b".to_string(),
        };
    }
    match noprop::sample_weighted_index(ctx, &[4, 3, 3, 1]) {
        0 => sample_term(ctx, 0),
        1 => {
            let n = noprop::sample_usize_in(ctx, 0..=MAX_CHILDREN);
            let mut parts = Vec::with_capacity(n);
            for _ in 0..n {
                parts.push(sample_term(ctx, depth - 1));
            }
            format!("{{{}}}", parts.join(", "))
        }
        2 => {
            let n = noprop::sample_usize_in(ctx, 0..=MAX_CHILDREN);
            let mut parts = Vec::with_capacity(n);
            for _ in 0..n {
                parts.push(sample_term(ctx, depth - 1));
            }
            format!("[{}]", parts.join(", "))
        }
        _ => {
            let inner = sample_term(ctx, depth - 1);
            format!("- {inner}")
        }
    }
}

/// Draws a term-list top-level source with N `.`-terminated terms.
pub fn sample_term_list_source(ctx: &mut noprop::TestCaseContext) -> String {
    let n =
        noprop::sample_with_boundaries(ctx, &[0usize, 1, 3], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 0..=3)
        });
    let mut out = String::new();
    for _ in 0..n {
        out.push_str(&sample_term(ctx, MAX_GEN_DEPTH));
        out.push_str(".\n");
    }
    out
}

/// Draws a module-mode source: N top-level forms.
pub fn sample_module_source(ctx: &mut noprop::TestCaseContext) -> String {
    let n =
        noprop::sample_with_boundaries(ctx, &[0usize, 1, 3], noprop::Ratio::one_nth(4), |ctx| {
            noprop::sample_usize_in(ctx, 0..=3)
        });
    let mut out = String::new();
    for _ in 0..n {
        out.push_str(&sample_form(ctx));
        out.push('\n');
    }
    out
}

/// Draws a single top-level form (attribute or function
/// declaration).
fn sample_form(ctx: &mut noprop::TestCaseContext) -> String {
    match noprop::sample_weighted_index(ctx, &[2, 3]) {
        0 => {
            // Attribute with or without parenthesized payload.
            let name = sample_atom_name(ctx);
            if noprop::sample_bool(ctx) {
                let payload = sample_expression(ctx, MAX_GEN_DEPTH - 2);
                format!("-{name}({payload}).")
            } else {
                format!("-{name}.")
            }
        }
        _ => {
            let name = sample_atom_name(ctx);
            let arity = noprop::sample_usize_in(ctx, 0..=2);
            let args: Vec<String> = (0..arity)
                .map(|_| sample_var_name(ctx).to_string())
                .collect();
            let body = sample_expression(ctx, MAX_GEN_DEPTH - 2);
            format!("{name}({}) -> {body}.", args.join(", "))
        }
    }
}

/// Draws a deeply nested paren expression that intentionally exceeds
/// `Parser::MAX_NESTING_DEPTH` so the depth-cap path is exercised.
pub fn sample_deep_paren_source(ctx: &mut noprop::TestCaseContext) -> String {
    let extra = noprop::sample_usize_in(ctx, 8..=64);
    let depth = Parser::MAX_NESTING_DEPTH + extra;
    let mut s = String::with_capacity(depth * 2 + 4);
    for _ in 0..depth {
        s.push('(');
    }
    s.push('1');
    for _ in 0..depth {
        s.push(')');
    }
    s.push('.');
    s
}

// -----------------------------------------------------------------
// Token utilities.
// -----------------------------------------------------------------

/// Runs `erl_tokenize::scan_token` end-to-end over `source`.
/// Returns `None` when the scanner rejects the input; the caller
/// then reject the case or picks another sample.
pub fn scan_all(source: &str) -> Option<Vec<Token>> {
    let mut out = Vec::new();
    let mut pos = Position::new();
    loop {
        match scan_token(source, pos) {
            Ok(Some(t)) => {
                let end = t.end();
                out.push(t);
                pos = end;
            }
            Ok(None) => return Some(out),
            Err(_) => return None,
        }
    }
}

/// Drives a parser end-to-end with the supplied tokens in the given
/// mode. Returns the finished `SyntaxTree`.
pub fn parse_full(mode: ParseMode, tokens: &[Token]) -> SyntaxTree {
    let mut p = Parser::new(mode);
    for t in tokens {
        p.push_token(*t);
    }
    p.finish()
}

// -----------------------------------------------------------------
// Validation helpers. Every helper takes `&SyntaxTree` (the public
// API) and inspects the borrow-only accessors — no test-only crate
// visibility.
// -----------------------------------------------------------------

/// Reports a specific invariant violation encountered while
/// walking the syntax index. Named variants keep failure messages
/// specific enough to bisect a broken generator or parser change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantViolation {
    /// A `TokenRange` extended past the token buffer's end.
    RangeBeyondTokens {
        node: NodeId,
        end: TokenIndex,
        buffer_len: usize,
    },
    /// A non-empty `TokenRange` had `start > end`.
    RangeInverted {
        node: NodeId,
        start: TokenIndex,
        end: TokenIndex,
    },
    /// An entry's `subtree_end` fell outside `self_index+1..=entries.len()`.
    SubtreeEndOutOfRange {
        node: NodeId,
        subtree_end: EntryIndex,
        self_index: usize,
        entries_len: usize,
    },
    /// A child subtree ended past its parent's `subtree_end`.
    ChildOverflowsParent {
        parent: NodeId,
        parent_subtree_end: EntryIndex,
        child: NodeId,
        child_subtree_end: EntryIndex,
    },
    /// A child's `TokenRange` was not contained in the parent's.
    ChildRangeOutsideParent {
        parent: NodeId,
        parent_range: TokenRange,
        child: NodeId,
        child_range: TokenRange,
    },
    /// Two adjacent errors with the same `(kind, start)` appear in
    /// `errors()` — the `push_unique_at_cursor` adjacent-dedupe
    /// contract was violated.
    AdjacentDuplicateError {
        first_idx: usize,
        kind: ParseErrorKind,
        start: TokenIndex,
    },
    /// A `SkippedToken` diagnostic did not have a matching
    /// `SyntaxKind::Error` node with the same `TokenRange`.
    SkippedTokenWithoutMatchingErrorNode { error_idx: usize, range: TokenRange },
    /// A `MissingToken` diagnostic carried a non-empty range.
    MissingTokenNotZeroWidth { error_idx: usize, range: TokenRange },
}

/// Runs every whole-tree invariant against `tree`. Returns
/// `Err(list)` on the first tree that violates at least one
/// invariant; `Ok(())` when the tree is clean.
pub fn validate_tree(tree: &SyntaxTree) -> Result<(), Vec<InvariantViolation>> {
    let mut violations = Vec::new();
    let syntax = tree.syntax();
    let tokens = tree.tokens();
    let buffer_len = tokens.len();

    // Range boundaries + subtree_end structural check.
    for i in 0..syntax.len() {
        let id = NodeId::new(i);
        let entry = syntax.entry(id).expect("id in bounds");
        let range = entry.range();
        if range.end().get() > buffer_len {
            violations.push(InvariantViolation::RangeBeyondTokens {
                node: id,
                end: range.end(),
                buffer_len,
            });
        }
        if range.start().get() > range.end().get() {
            violations.push(InvariantViolation::RangeInverted {
                node: id,
                start: range.start(),
                end: range.end(),
            });
        }
        let se = entry.subtree_end();
        if se.get() < i + 1 || se.get() > syntax.len() {
            violations.push(InvariantViolation::SubtreeEndOutOfRange {
                node: id,
                subtree_end: se,
                self_index: i,
                entries_len: syntax.len(),
            });
        }
    }

    // Preorder containment: iterate parent + children.
    for i in 0..syntax.len() {
        let parent_id = NodeId::new(i);
        let parent = syntax.entry(parent_id).expect("id in bounds");
        let parent_end = parent.subtree_end().get();
        let mut j = i + 1;
        while j < parent_end {
            let child_id = NodeId::new(j);
            let child = syntax.entry(child_id).expect("child in bounds");
            let child_end = child.subtree_end();
            if child_end.get() > parent_end {
                violations.push(InvariantViolation::ChildOverflowsParent {
                    parent: parent_id,
                    parent_subtree_end: parent.subtree_end(),
                    child: child_id,
                    child_subtree_end: child_end,
                });
                break;
            }
            let pr = parent.range();
            let cr = child.range();
            if cr.start().get() < pr.start().get() || cr.end().get() > pr.end().get() {
                violations.push(InvariantViolation::ChildRangeOutsideParent {
                    parent: parent_id,
                    parent_range: pr,
                    child: child_id,
                    child_range: cr,
                });
            }
            j = child_end.get();
        }
    }

    // Adjacent-dedupe invariant on errors.
    let errs = tree.errors();
    for w in errs.windows(2).enumerate() {
        let (i, pair) = w;
        let a = pair[0];
        let b = pair[1];
        if a.kind() == b.kind() && a.range().start() == b.range().start() {
            violations.push(InvariantViolation::AdjacentDuplicateError {
                first_idx: i,
                kind: a.kind(),
                start: a.range().start(),
            });
        }
    }

    // SkippedToken / MissingToken contracts.
    for (idx, err) in errs.iter().enumerate() {
        match err.kind() {
            ParseErrorKind::SkippedToken => {
                let matches_node = syntax
                    .entries()
                    .iter()
                    .any(|e| e.kind() == SyntaxKind::Error && e.range() == err.range());
                if !matches_node {
                    violations.push(InvariantViolation::SkippedTokenWithoutMatchingErrorNode {
                        error_idx: idx,
                        range: err.range(),
                    });
                }
            }
            ParseErrorKind::MissingToken if !err.range().is_empty() => {
                violations.push(InvariantViolation::MissingTokenNotZeroWidth {
                    error_idx: idx,
                    range: err.range(),
                });
            }
            _ => {}
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Confirms that `tree.tokens()` byte-for-byte equals `expected`.
/// The parser must never add, remove, or reorder input tokens.
pub fn assert_tokens_unchanged(tokens: &TokenBuffer, expected: &[Token]) {
    assert_eq!(
        tokens.len(),
        expected.len(),
        "parser modified token buffer length"
    );
    for (i, exp) in expected.iter().enumerate() {
        let got = tokens.get(TokenIndex::new(i)).expect("index in range");
        assert_eq!(got, *exp, "parser modified token at index {i}");
    }
}

// -----------------------------------------------------------------
// Fake fixtures — for the negative test that the validation
// helper actually rejects a broken tree. We cannot construct a
// bad `SyntaxIndex` through the production API (constructors are
// `pub(crate)`); instead we shape a small `impl` around the two
// invariants exposed here and probe each by hand-crafted small
// examples.
// -----------------------------------------------------------------

/// Runs the adjacent-dedupe invariant on a synthetic error list to
/// confirm the helper's logic rejects a broken input.
pub fn adjacent_dedupe_check(errors: &[ParseError]) -> Result<(), usize> {
    for (i, pair) in errors.windows(2).enumerate() {
        if pair[0].kind() == pair[1].kind() && pair[0].range().start() == pair[1].range().start() {
            return Err(i);
        }
    }
    Ok(())
}

/// Runs the `MissingToken` zero-width invariant on a synthetic
/// error to confirm the helper rejects a non-empty range under
/// `MissingToken` kind.
pub fn missing_zero_width_check(err: ParseError) -> Result<(), TokenRange> {
    if err.kind() == ParseErrorKind::MissingToken && !err.range().is_empty() {
        return Err(err.range());
    }
    Ok(())
}

// -----------------------------------------------------------------
// Negative tests: verify the validation helpers actually reject a
// broken input. `SyntaxIndex` cannot be corrupted from an
// integration test (its `push` is `pub(crate)`), so we probe only
// the small helpers here on hand-crafted `ParseError`s. `validate_tree`
// itself is exercised on real parser output by the `pbt_syntax_index`
// tests — if the helper missed a real bug, those tests would let
// the bug through.
// -----------------------------------------------------------------

#[test]
fn adjacent_dedupe_check_rejects_adjacent_pair_with_same_kind_and_start() {
    let at = TokenIndex::new(3);
    let mk = |k: ParseErrorKind| {
        ParseError::new(
            k,
            TokenRange::empty_at(at),
            erl_parse::Expected::Unspecified,
            None,
        )
    };
    let bad = [
        mk(ParseErrorKind::MissingToken),
        mk(ParseErrorKind::MissingToken),
    ];
    let result = adjacent_dedupe_check(&bad);
    assert_eq!(
        result,
        Err(0),
        "helper must reject an adjacent duplicate pair at index 0"
    );
}

#[test]
fn adjacent_dedupe_check_accepts_different_kinds_at_same_start() {
    let at = TokenIndex::new(3);
    let mk = |k: ParseErrorKind| {
        ParseError::new(
            k,
            TokenRange::empty_at(at),
            erl_parse::Expected::Unspecified,
            None,
        )
    };
    let ok = [
        mk(ParseErrorKind::MissingToken),
        mk(ParseErrorKind::UnexpectedToken),
    ];
    assert_eq!(adjacent_dedupe_check(&ok), Ok(()));
}

#[test]
fn adjacent_dedupe_check_accepts_non_adjacent_duplicates() {
    let at = TokenIndex::new(3);
    let other = TokenIndex::new(9);
    let mk = |k: ParseErrorKind, i: TokenIndex| {
        ParseError::new(
            k,
            TokenRange::empty_at(i),
            erl_parse::Expected::Unspecified,
            None,
        )
    };
    // Same (kind, start) at positions 0 and 2 with a different
    // error at position 1 — non-adjacent, so the invariant holds.
    let ok = [
        mk(ParseErrorKind::MissingToken, at),
        mk(ParseErrorKind::UnexpectedToken, other),
        mk(ParseErrorKind::MissingToken, at),
    ];
    assert_eq!(adjacent_dedupe_check(&ok), Ok(()));
}

#[test]
fn missing_zero_width_check_rejects_non_empty_missing_range() {
    let start = TokenIndex::new(1);
    let end = TokenIndex::new(4);
    let bad_range = TokenRange::new(start, end);
    let bad = ParseError::new(
        ParseErrorKind::MissingToken,
        bad_range,
        erl_parse::Expected::Unspecified,
        None,
    );
    assert_eq!(missing_zero_width_check(bad), Err(bad_range));
}

#[test]
fn missing_zero_width_check_accepts_zero_width_missing() {
    let at = TokenIndex::new(2);
    let ok = ParseError::new(
        ParseErrorKind::MissingToken,
        TokenRange::empty_at(at),
        erl_parse::Expected::Unspecified,
        None,
    );
    assert_eq!(missing_zero_width_check(ok), Ok(()));
}

#[test]
fn missing_zero_width_check_ignores_other_kinds() {
    // A non-empty range under a different kind is not a
    // MissingToken violation.
    let bad_range = TokenRange::new(TokenIndex::new(1), TokenIndex::new(3));
    let other = ParseError::new(
        ParseErrorKind::SkippedToken,
        bad_range,
        erl_parse::Expected::Unspecified,
        None,
    );
    assert_eq!(missing_zero_width_check(other), Ok(()));
}
