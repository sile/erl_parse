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
/// `erl_parse::Parser::MAX_NESTING_DEPTH` on purpose for the cap-check test.
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

/// Draws a type-expression source of bounded depth. Leaves are atoms,
/// integers, nullary type calls (`atom()`), and variables; compounds
/// include tuples, lists, unions, integer ranges, and parentheses.
pub fn sample_type(ctx: &mut noprop::TestCaseContext, depth: usize) -> String {
    if depth == 0 {
        return match noprop::sample_weighted_index(ctx, &[3, 2, 2, 1]) {
            0 => sample_atom_name(ctx).to_string(),
            1 => sample_integer_literal(ctx),
            2 => format!("{}()", sample_atom_name(ctx)),
            _ => sample_var_name(ctx).to_string(),
        };
    }
    match noprop::sample_weighted_index(ctx, &[3, 2, 2, 2, 1, 1]) {
        0 => sample_type(ctx, 0),
        1 => {
            let n = noprop::sample_usize_in(ctx, 0..=MAX_CHILDREN);
            let mut parts = Vec::with_capacity(n);
            for _ in 0..n {
                parts.push(sample_type(ctx, depth - 1));
            }
            format!("{{{}}}", parts.join(", "))
        }
        2 => {
            let inner = sample_type(ctx, depth - 1);
            format!("[{inner}]")
        }
        3 => {
            let lhs = sample_type(ctx, depth - 1);
            let rhs = sample_type(ctx, depth - 1);
            format!("{lhs} | {rhs}")
        }
        4 => {
            let lo = sample_integer_literal(ctx);
            let hi = sample_integer_literal(ctx);
            format!("{lo}..{hi}")
        }
        _ => {
            let inner = sample_type(ctx, depth - 1);
            format!("({inner})")
        }
    }
}

/// Wraps [`sample_type`] with a terminating `.` so the result is a
/// valid type-mode top-level unit.
pub fn sample_type_unit(ctx: &mut noprop::TestCaseContext) -> String {
    let mut s = sample_type(ctx, MAX_GEN_DEPTH);
    s.push('.');
    s
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
/// `erl_parse::Parser::MAX_NESTING_DEPTH` so the depth-cap path is exercised.
pub fn sample_deep_paren_source(ctx: &mut noprop::TestCaseContext) -> String {
    let extra = noprop::sample_usize_in(ctx, 8..=64);
    let depth = erl_parse::Parser::MAX_NESTING_DEPTH + extra;
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
// erl_tokenize::Token utilities.
// -----------------------------------------------------------------

/// Runs `erl_tokenize::scan_token` end-to-end over `source`.
/// Returns `None` when the scanner rejects the input; the caller
/// then reject the case or picks another sample.
pub fn scan_all(source: &str) -> Option<Vec<erl_tokenize::Token>> {
    let mut out = Vec::new();
    let mut pos = erl_tokenize::Position::new();
    loop {
        match erl_tokenize::scan_token(source, pos) {
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
/// mode. Returns the finished `erl_parse::SyntaxTree`.
pub fn parse_full(
    mode: erl_parse::ParseMode,
    tokens: &[erl_tokenize::Token],
) -> erl_parse::SyntaxTree {
    let mut p = erl_parse::Parser::new(mode);
    for t in tokens {
        p.feed_token(*t);
    }
    p.finish()
}

/// All public [`erl_parse::ParseMode`] variants, in a stable order
/// used by properties that sample a mode then a matching source.
pub const ALL_MODES: &[erl_parse::ParseMode] = &[
    erl_parse::ParseMode::Expression,
    erl_parse::ParseMode::Module,
    erl_parse::ParseMode::TermList,
    erl_parse::ParseMode::Type,
];

/// Draws a top-level source appropriate for `mode`.
pub fn sample_source_for_mode(
    ctx: &mut noprop::TestCaseContext,
    mode: erl_parse::ParseMode,
) -> String {
    match mode {
        erl_parse::ParseMode::Expression => sample_expression_unit(ctx),
        erl_parse::ParseMode::Module => sample_module_source(ctx),
        erl_parse::ParseMode::TermList => sample_term_list_source(ctx),
        erl_parse::ParseMode::Type => sample_type_unit(ctx),
    }
}

/// Short label for coverage gates that distinguish modes.
pub fn mode_label(mode: erl_parse::ParseMode) -> &'static str {
    match mode {
        erl_parse::ParseMode::Expression => "expression",
        erl_parse::ParseMode::Module => "module",
        erl_parse::ParseMode::TermList => "term-list",
        erl_parse::ParseMode::Type => "type",
    }
}

// -----------------------------------------------------------------
// Validation helpers. Every helper takes `&erl_parse::SyntaxTree` (the public
// API) and inspects the borrow-only accessors — no test-only crate
// visibility.
// -----------------------------------------------------------------

/// Reports a specific invariant violation encountered while
/// walking the syntax index. Named variants keep failure messages
/// specific enough to bisect a broken generator or parser change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantViolation {
    /// A `erl_parse::TokenRange` extended past the token buffer's end.
    RangeBeyondTokens {
        end: erl_parse::TokenIndex,
        buffer_len: usize,
    },
    /// A non-empty `erl_parse::TokenRange` had `start > end`.
    RangeInverted {
        start: erl_parse::TokenIndex,
        end: erl_parse::TokenIndex,
    },
    /// A child's `erl_parse::TokenRange` was not contained in the parent's.
    ChildRangeOutsideParent {
        parent: erl_parse::NodeId,
        parent_range: erl_parse::TokenRange,
        child: erl_parse::NodeId,
        child_range: erl_parse::TokenRange,
    },
    /// An index slot was reached more than once from the root walk.
    ReachedTwice { node: erl_parse::NodeId },
    /// Two adjacent diagnostics with the same `(kind, start)` appear in
    /// `diagnostics()` — the `push_unique_at_cursor` adjacent-dedupe
    /// contract was violated.
    AdjacentDuplicateError {
        first_idx: usize,
        kind: erl_parse::DiagnosticKind,
        start: erl_parse::TokenIndex,
    },
    /// A `SkippedToken` diagnostic did not have a matching
    /// `erl_parse::SyntaxKind::Error` node with the same `erl_parse::TokenRange`.
    SkippedTokenWithoutMatchingErrorNode {
        error_idx: usize,
        range: erl_parse::TokenRange,
    },
    /// A `MissingToken` diagnostic carried a non-empty range.
    MissingTokenNotZeroWidth {
        error_idx: usize,
        range: erl_parse::TokenRange,
    },
}

/// Walks every node through the public forest: each root, then its
/// descendants.
pub fn all_views<'a>(
    tree: &'a erl_parse::SyntaxTree,
) -> impl Iterator<Item = erl_parse::NodeView<'a>> {
    tree.roots()
        .flat_map(|root| std::iter::once(root).chain(root.descendants()))
}

/// Number of nodes reachable from the forest roots.
pub fn node_count(tree: &erl_parse::SyntaxTree) -> usize {
    all_views(tree).count()
}

/// Preorder `(kind, range)` pairs of every reachable node.
pub fn preorder_kind_and_range(
    tree: &erl_parse::SyntaxTree,
) -> Vec<(erl_parse::SyntaxKind, erl_parse::TokenRange)> {
    all_views(tree).map(|v| (v.kind(), v.range())).collect()
}

/// Runs every whole-tree invariant against `tree`. Returns
/// `Err(list)` on the first tree that violates at least one
/// invariant; `Ok(())` when the tree is clean.
pub fn validate_tree(tree: &erl_parse::SyntaxTree) -> Result<(), Vec<InvariantViolation>> {
    let mut violations = Vec::new();
    let tokens = tree.tokens();
    let buffer_len = tokens.len();

    // Range boundaries. Walked through the public forest rather than
    // the crate-internal index slice.
    for node in all_views(tree) {
        let range = node.range();
        if range.end().get() > buffer_len {
            violations.push(InvariantViolation::RangeBeyondTokens {
                end: range.end(),
                buffer_len,
            });
        }
        if range.start().get() > range.end().get() {
            violations.push(InvariantViolation::RangeInverted {
                start: range.start(),
                end: range.end(),
            });
        }
    }

    // Child token ranges sit inside the parent. Walked through the
    // public `children` iterator rather than the preorder fence.
    for parent in all_views(tree) {
        let pr = parent.range();
        for child in parent.children() {
            let cr = child.range();
            if cr.start().get() < pr.start().get() || cr.end().get() > pr.end().get() {
                violations.push(InvariantViolation::ChildRangeOutsideParent {
                    parent: parent.node_id(),
                    parent_range: pr,
                    child: child.node_id(),
                    child_range: cr,
                });
            }
        }
    }

    // The public forest walk visits each node at most once.
    let mut seen = BTreeSet::new();
    let mut twice = Vec::new();
    for node in all_views(tree) {
        let id = node.node_id();
        if !seen.insert(id) {
            twice.push(id);
        }
    }
    for node in twice {
        violations.push(InvariantViolation::ReachedTwice { node });
    }

    // Adjacent-dedupe invariant on diagnostics.
    let errs = tree.diagnostics();
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
            erl_parse::DiagnosticKind::SkippedToken => {
                let matches_node = all_views(tree)
                    .any(|v| v.kind() == erl_parse::SyntaxKind::Error && v.range() == err.range());
                if !matches_node {
                    violations.push(InvariantViolation::SkippedTokenWithoutMatchingErrorNode {
                        error_idx: idx,
                        range: err.range(),
                    });
                }
            }
            erl_parse::DiagnosticKind::MissingToken if !err.range().is_empty() => {
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
pub fn assert_tokens_unchanged(tokens: &[erl_tokenize::Token], expected: &[erl_tokenize::Token]) {
    assert_eq!(tokens, expected, "parser modified the input token sequence");
}
