//! Property-based tests for the syntax-index structural
//! invariants: range boundaries, preorder containment, subtree
//! boundaries, and that the parser never modifies the input token
//! buffer.

#[expect(dead_code, reason = "shared harness; this binary uses only a subset")]
mod pbt_harness;

use erl_parse::ParseMode;
use pbt_harness::{
    CASES, Counter, Flag, LabelSet, SEED_ENV, assert_tokens_unchanged, parse_full,
    sample_expression_unit, sample_module_source, sample_term_list_source, scan_all, validate_tree,
};

/// For every valid expression source we generate, the resulting
/// `SyntaxTree` satisfies every structural invariant `validate_tree`
/// checks (range boundaries, preorder containment, subtree boundary,
/// SkippedToken/MissingToken contracts).
#[test]
fn expression_mode_tree_invariants_hold() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let non_empty = Counter::new();
    let nested = Flag::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(CASES, |ctx| {
        let src = sample_expression_unit(ctx);
        let Some(tokens) = scan_all(&src) else {
            return Ok(());
        };
        let tree = parse_full(ParseMode::Expression, &tokens);
        if let Err(violations) = validate_tree(&tree) {
            panic!("invariant violations for source {src:?}: {violations:?}");
        }
        if tree.syntax().len() > 1 {
            non_empty.bump();
        }
        if tree.syntax().len() >= 3 {
            nested.set();
        }
        Ok(())
    })?;
    assert!(
        non_empty.get() > 0,
        "no case produced a non-trivial syntax tree\n{runner}"
    );
    assert!(
        nested.hit(),
        "no case produced a nested (>=3 entries) tree\n{runner}"
    );
    Ok(())
}

/// Same invariants hold for module-mode sources (attributes and
/// function declarations mixed).
#[test]
fn module_mode_tree_invariants_hold() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let saw_form = Counter::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(CASES, |ctx| {
        let src = sample_module_source(ctx);
        let Some(tokens) = scan_all(&src) else {
            return Ok(());
        };
        let tree = parse_full(ParseMode::Module, &tokens);
        if let Err(violations) = validate_tree(&tree) {
            panic!("invariant violations for source {src:?}: {violations:?}");
        }
        if !tree.syntax().is_empty() {
            saw_form.bump();
        }
        Ok(())
    })?;
    assert!(
        saw_form.get() > 0,
        "no case produced any form at module top level\n{runner}"
    );
    Ok(())
}

/// Same invariants hold for term-list-mode sources.
#[test]
fn term_list_mode_tree_invariants_hold() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let saw_term = Counter::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(CASES, |ctx| {
        let src = sample_term_list_source(ctx);
        let Some(tokens) = scan_all(&src) else {
            return Ok(());
        };
        let tree = parse_full(ParseMode::TermList, &tokens);
        if let Err(violations) = validate_tree(&tree) {
            panic!("invariant violations for source {src:?}: {violations:?}");
        }
        if !tree.syntax().is_empty() {
            saw_term.bump();
        }
        Ok(())
    })?;
    assert!(
        saw_term.get() > 0,
        "no case produced any term at term-list top level\n{runner}"
    );
    Ok(())
}

/// For any generator-produced token stream, the parser leaves the
/// input token buffer unchanged: same length, same tokens in the
/// same positions.
#[test]
fn parser_never_modifies_the_input_token_buffer() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);
    let modes = &[
        ParseMode::Expression,
        ParseMode::Module,
        ParseMode::TermList,
    ];
    let saw_each_mode = LabelSet::new();
    runner.run(CASES, |ctx| {
        let mode = noprop::sample_choice(ctx, modes);
        let src = match mode {
            ParseMode::Expression => sample_expression_unit(ctx),
            ParseMode::Module => sample_module_source(ctx),
            ParseMode::TermList => sample_term_list_source(ctx),
        };
        let Some(tokens) = scan_all(&src) else {
            return Ok(());
        };
        let tree = parse_full(mode, &tokens);
        assert_tokens_unchanged(tree.tokens(), &tokens);
        saw_each_mode.insert(match mode {
            ParseMode::Expression => "expression",
            ParseMode::Module => "module",
            ParseMode::TermList => "term-list",
        });
        Ok(())
    })?;
    for label in ["expression", "module", "term-list"] {
        assert!(
            saw_each_mode.contains(label),
            "no case exercised mode {label}\n{runner}"
        );
    }
    Ok(())
}
