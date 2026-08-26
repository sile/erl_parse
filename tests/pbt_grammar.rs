//! Property-based tests that exercise the parser at the mode /
//! entry-point level: determinism and termination.

#[expect(dead_code, reason = "shared harness; this binary uses only a subset")]
mod pbt_harness;

/// Two calls to [`erl_parse::parse`] with the same tokens in the same
/// mode produce matching preorder `(kind, range)` sequences and
/// `erl_parse::Diagnostic` sequences.
#[test]
fn determinism_across_two_parsers() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);
    runner.run(pbt_harness::CASES, |ctx| {
        let mode = noprop::sample_choice(ctx, pbt_harness::ALL_MODES);
        let src = pbt_harness::sample_source_for_mode(ctx, mode);
        let Some(tokens) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let ta = erl_parse::parse(tokens.as_slice(), mode);
        let tb = erl_parse::parse(tokens.as_slice(), mode);
        assert_eq!(
            pbt_harness::preorder_kind_and_range(&ta),
            pbt_harness::preorder_kind_and_range(&tb),
            "syntax trees disagree for source {src:?}"
        );
        assert_eq!(
            ta.diagnostics(),
            tb.diagnostics(),
            "errors disagree for source {src:?}"
        );
        Ok(())
    })?;
    Ok(())
}

/// For any input in any of the four modes, [`erl_parse::parse`]
/// returns without panicking or hanging: parsing terminates.
#[test]
fn parser_always_terminates_across_modes() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let touched = pbt_harness::Flag::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(pbt_harness::CASES, |ctx| {
        let mode = noprop::sample_choice(ctx, pbt_harness::ALL_MODES);
        let src = pbt_harness::sample_source_for_mode(ctx, mode);
        let Some(tokens) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let _tree = erl_parse::parse(tokens, mode);
        touched.set();
        Ok(())
    })?;
    assert!(
        touched.hit(),
        "no case exercised parser termination\n{runner}"
    );
    Ok(())
}
