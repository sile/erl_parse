//! Property-based tests for the parser's error-recovery
//! contracts: `SkippedToken` range equals its `erl_parse::SyntaxKind::Error`
//! node's `erl_parse::TokenRange`; `MissingToken` is zero-width and never
//! fabricates a `erl_tokenize::Token`; adjacent-dedupe holds; the depth cap
//! surfaces as `NestingDepthExceeded` without stack-overflowing.

#[expect(dead_code, reason = "shared harness; this binary uses only a subset")]
mod pbt_harness;

/// Randomly deletes lexical tokens from a valid source's scan
/// result — a cheap way to reach the recovery paths without hand-
/// crafted fixtures. Deletes at most `n` tokens.
fn mutate_delete(
    ctx: &mut noprop::TestCaseContext,
    tokens: &[erl_tokenize::Token],
    max_deletions: usize,
) -> Vec<erl_tokenize::Token> {
    let mut out: Vec<_> = tokens.to_vec();
    if out.is_empty() {
        return out;
    }
    let deletions = noprop::sample_usize_in(ctx, 0..=max_deletions);
    for _ in 0..deletions {
        if out.is_empty() {
            break;
        }
        let idx = noprop::sample_usize_in(ctx, 0..out.len());
        out.remove(idx);
    }
    out
}

/// Randomly duplicates lexical tokens from a valid source's scan
/// result. Duplicates at most `n` positions.
fn mutate_duplicate(
    ctx: &mut noprop::TestCaseContext,
    tokens: &[erl_tokenize::Token],
    max_dupes: usize,
) -> Vec<erl_tokenize::Token> {
    let mut out: Vec<_> = tokens.to_vec();
    if out.is_empty() {
        return out;
    }
    let dupes = noprop::sample_usize_in(ctx, 0..=max_dupes);
    for _ in 0..dupes {
        let idx = noprop::sample_usize_in(ctx, 0..out.len());
        let t = out[idx];
        out.insert(idx, t);
    }
    out
}

/// Tree-level invariants hold for mutated (grammar-invalid) input.
/// This is the composite check: SkippedToken / MissingToken
/// contracts, subtree structure, adjacent-dedupe, range boundaries
/// all pass on recovery-path outputs.
#[test]
fn tree_invariants_hold_for_mutated_input() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let saw_errors = pbt_harness::Counter::new();
    let saw_skipped = pbt_harness::Flag::new();
    let saw_missing = pbt_harness::Flag::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(pbt_harness::CASES, |ctx| {
        let src = pbt_harness::sample_module_source(ctx);
        let Some(scanned) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let mutated = match noprop::sample_weighted_index(ctx, &[3, 2, 1]) {
            0 => mutate_delete(ctx, &scanned, 3),
            1 => mutate_duplicate(ctx, &scanned, 2),
            _ => scanned.clone(),
        };
        let tree = pbt_harness::parse_full(erl_parse::ParseMode::Module, &mutated);
        if let Err(violations) = pbt_harness::validate_tree(&tree) {
            panic!(
                "invariant violations for source {src:?} mutated to {n} tokens: {violations:?}",
                n = mutated.len()
            );
        }
        if !tree.diagnostics().is_empty() {
            saw_errors.bump();
            for e in tree.diagnostics() {
                if e.kind() == erl_parse::DiagnosticKind::SkippedToken {
                    saw_skipped.set();
                }
                if e.kind() == erl_parse::DiagnosticKind::MissingToken {
                    saw_missing.set();
                }
            }
        }
        Ok(())
    })?;
    assert!(
        saw_errors.get() > 0,
        "no case exercised the recovery path\n{runner}"
    );
    assert!(
        saw_skipped.hit(),
        "no case produced a SkippedToken diagnostic\n{runner}"
    );
    assert!(
        saw_missing.hit(),
        "no case produced a MissingToken diagnostic\n{runner}"
    );
    Ok(())
}

/// `SkippedToken` diagnostics always have a matching
/// `erl_parse::SyntaxKind::Error` node with the same `erl_parse::TokenRange`. This is a
/// re-statement of one clause in `pbt_harness::validate_tree`, kept as a
/// standalone test so the failing property message points at this
/// specific invariant directly.
#[test]
fn skipped_token_range_matches_error_node_range() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let saw = pbt_harness::Counter::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(pbt_harness::CASES, |ctx| {
        let src = pbt_harness::sample_module_source(ctx);
        let Some(scanned) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let mutated = mutate_delete(ctx, &scanned, 3);
        let tree = pbt_harness::parse_full(erl_parse::ParseMode::Module, &mutated);
        for err in tree.diagnostics() {
            if err.kind() != erl_parse::DiagnosticKind::SkippedToken {
                continue;
            }
            let matched = tree
                .syntax()
                .entries()
                .iter()
                .any(|e| e.kind() == erl_parse::SyntaxKind::Error && e.range() == err.range());
            assert!(
                matched,
                "SkippedToken with range {:?} has no matching Error node in tree for {src:?}",
                err.range()
            );
            saw.bump();
        }
        Ok(())
    })?;
    assert!(
        saw.get() > 0,
        "no case observed a SkippedToken diagnostic\n{runner}"
    );
    Ok(())
}

/// `MissingToken` diagnostics never come with an added
/// `erl_tokenize::Token` in the buffer: pushed token count equals
/// `tree.tokens().len()`.
#[test]
fn missing_token_does_not_fabricate_tokens() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let saw = pbt_harness::Counter::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(pbt_harness::CASES, |ctx| {
        let src = pbt_harness::sample_module_source(ctx);
        let Some(scanned) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let mutated = mutate_delete(ctx, &scanned, 3);
        let pushed = mutated.len();
        let tree = pbt_harness::parse_full(erl_parse::ParseMode::Module, &mutated);
        assert_eq!(
            tree.tokens().len(),
            pushed,
            "parser fabricated a token for MissingToken (pushed={pushed}, buffer={}); source {src:?}",
            tree.tokens().len(),
        );
        for e in tree.diagnostics() {
            if e.kind() == erl_parse::DiagnosticKind::MissingToken {
                assert!(
                    e.range().is_empty(),
                    "MissingToken has non-empty range {:?} in {src:?}",
                    e.range()
                );
                saw.bump();
            }
        }
        Ok(())
    })?;
    assert!(
        saw.get() > 0,
        "no case observed a MissingToken diagnostic\n{runner}"
    );
    Ok(())
}

/// Consecutive `erl_parse::Diagnostic`s never share `(kind, range().start())`.
/// Non-consecutive repetition is legal — this is the adjacent
/// dedupe contract of `push_unique_at_cursor`.
#[test]
fn adjacent_dedupe_holds_across_mutations() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let saw_errors = pbt_harness::Counter::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(pbt_harness::CASES, |ctx| {
        let src = pbt_harness::sample_module_source(ctx);
        let Some(scanned) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let mutated = mutate_delete(ctx, &scanned, 3);
        let tree = pbt_harness::parse_full(erl_parse::ParseMode::Module, &mutated);
        let errs = tree.diagnostics();
        for (i, pair) in errs.windows(2).enumerate() {
            let a = pair[0];
            let b = pair[1];
            assert!(
                !(a.kind() == b.kind() && a.range().start() == b.range().start()),
                "adjacent duplicate error at index {i}: {a:?} then {b:?}; source {src:?}"
            );
        }
        if !errs.is_empty() {
            saw_errors.bump();
        }
        Ok(())
    })?;
    assert!(
        saw_errors.get() > 0,
        "no case surfaced errors to check adjacent dedupe on\n{runner}"
    );
    Ok(())
}

/// Inputs whose nesting exceeds `erl_parse::Parser::MAX_NESTING_DEPTH` produce
/// a `NestingDepthExceeded` diagnostic instead of overflowing the
/// stack.
#[test]
fn depth_cap_surfaces_as_structured_error() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let hits = pbt_harness::Counter::new();
    let mut runner = noprop::Runner::new(seed);
    // Reduce case count since the deep-paren generator produces
    // large inputs; still enough to trigger the cap reliably.
    runner.run(32, |ctx| {
        let src = pbt_harness::sample_deep_paren_source(ctx);
        let Some(tokens) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let tree = pbt_harness::parse_full(erl_parse::ParseMode::Expression, &tokens);
        assert!(
            tree.diagnostics()
                .iter()
                .any(|e| e.kind() == erl_parse::DiagnosticKind::NestingDepthExceeded),
            "deep-paren source of length {} produced no NestingDepthExceeded",
            src.len()
        );
        hits.bump();
        // Sanity: the diagnostic anchors at a valid erl_parse::TokenIndex.
        for e in tree.diagnostics() {
            if e.kind() == erl_parse::DiagnosticKind::NestingDepthExceeded {
                let end: erl_parse::TokenIndex = tree.tokens().end_index();
                assert!(e.range().start().get() <= end.get());
            }
        }
        Ok(())
    })?;
    assert!(
        hits.get() > 0,
        "no case ran the depth-cap generator\n{runner}"
    );
    // The public contract is a fixed value at test time.
    assert_eq!(erl_parse::Parser::MAX_NESTING_DEPTH, 256);
    Ok(())
}
