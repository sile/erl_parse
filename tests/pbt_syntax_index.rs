//! Property-based tests for the syntax-index structural
//! invariants: range boundaries, preorder containment, subtree
//! boundaries, and that the parser never modifies the input token
//! buffer.

#[expect(dead_code, reason = "shared harness; this binary uses only a subset")]
mod pbt_harness;

/// For every valid expression source we generate, the resulting
/// `erl_parse::SyntaxTree` satisfies every structural invariant `pbt_harness::validate_tree`
/// checks (range boundaries, preorder containment, subtree boundary,
/// SkippedToken/MissingToken contracts).
#[test]
fn expression_mode_tree_invariants_hold() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let non_empty = pbt_harness::Counter::new();
    let nested = pbt_harness::Flag::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(pbt_harness::CASES, |ctx| {
        let src = pbt_harness::sample_expression_unit(ctx);
        let Some(tokens) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let tree = pbt_harness::parse_full(erl_parse::ParseMode::Expression, &tokens);
        if let Err(violations) = pbt_harness::validate_tree(&tree) {
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
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let saw_form = pbt_harness::Counter::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(pbt_harness::CASES, |ctx| {
        let src = pbt_harness::sample_module_source(ctx);
        let Some(tokens) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let tree = pbt_harness::parse_full(erl_parse::ParseMode::Module, &tokens);
        if let Err(violations) = pbt_harness::validate_tree(&tree) {
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
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let saw_term = pbt_harness::Counter::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(pbt_harness::CASES, |ctx| {
        let src = pbt_harness::sample_term_list_source(ctx);
        let Some(tokens) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let tree = pbt_harness::parse_full(erl_parse::ParseMode::TermList, &tokens);
        if let Err(violations) = pbt_harness::validate_tree(&tree) {
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
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);
    let modes = &[
        erl_parse::ParseMode::Expression,
        erl_parse::ParseMode::Module,
        erl_parse::ParseMode::TermList,
    ];
    let saw_each_mode = pbt_harness::LabelSet::new();
    runner.run(pbt_harness::CASES, |ctx| {
        let mode = noprop::sample_choice(ctx, modes);
        let src = match mode {
            erl_parse::ParseMode::Expression => pbt_harness::sample_expression_unit(ctx),
            erl_parse::ParseMode::Module => pbt_harness::sample_module_source(ctx),
            erl_parse::ParseMode::TermList => pbt_harness::sample_term_list_source(ctx),
        };
        let Some(tokens) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let tree = pbt_harness::parse_full(mode, &tokens);
        pbt_harness::assert_tokens_unchanged(tree.tokens(), &tokens);
        saw_each_mode.insert(match mode {
            erl_parse::ParseMode::Expression => "expression",
            erl_parse::ParseMode::Module => "module",
            erl_parse::ParseMode::TermList => "term-list",
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
