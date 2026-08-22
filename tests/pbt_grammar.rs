//! Property-based tests that exercise the parser at the mode /
//! entry-point level: determinism and observation-invariance.

#[expect(dead_code, reason = "shared harness; this binary uses only a subset")]
mod pbt_harness;

/// Two fresh `erl_parse::Parser` instances driven with the same tokens in the
/// same order produce byte-identical `SyntaxIndex` entries and
/// `erl_parse::ParseError` sequences.
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
        let mut a = erl_parse::Parser::new(mode);
        let mut b = erl_parse::Parser::new(mode);
        for t in &tokens {
            a.push_token(*t);
            b.push_token(*t);
        }
        let ta = a.finish();
        let tb = b.finish();
        assert_eq!(
            ta.syntax().entries(),
            tb.syntax().entries(),
            "syntax indexes disagree for source {src:?}"
        );
        assert_eq!(
            ta.errors(),
            tb.errors(),
            "errors disagree for source {src:?}"
        );
        Ok(())
    })?;
    Ok(())
}

/// Two parsers driven with the same tokens produce the same
/// `SyntaxIndex` / `erl_parse::ParseError` regardless of whether the caller
/// interleaves `next_top_node` / `state` / `syntax_tree`
/// observations between `push_token` calls or defers all
/// observation until `finish` returns.
#[test]
fn observation_invariance() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(pbt_harness::SEED_ENV)?;
    let saw_observation = pbt_harness::Counter::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(pbt_harness::CASES, |ctx| {
        let src = pbt_harness::sample_module_source(ctx);
        let Some(tokens) = pbt_harness::scan_all(&src) else {
            return Ok(());
        };
        let mut quiet = erl_parse::Parser::new(erl_parse::ParseMode::Module);
        let mut noisy = erl_parse::Parser::new(erl_parse::ParseMode::Module);
        for (i, t) in tokens.iter().enumerate() {
            quiet.push_token(*t);
            noisy.push_token(*t);
            // Interleave observations at pseudo-random points.
            if i.is_multiple_of(3) {
                let _ = noisy.next_top_node();
                let _ = noisy.state();
                let _ = noisy.syntax_tree();
                saw_observation.bump();
            }
        }
        // Drain any remaining pending units on the noisy side.
        while noisy.next_top_node().is_some() {}
        let tq = quiet.finish();
        let tn = noisy.finish();
        assert_eq!(
            tq.syntax().entries(),
            tn.syntax().entries(),
            "observation altered syntax entries for source {src:?}"
        );
        assert_eq!(
            tq.errors(),
            tn.errors(),
            "observation altered errors for source {src:?}"
        );
        Ok(())
    })?;
    assert!(
        saw_observation.get() > 0,
        "no case interleaved observation calls\n{runner}"
    );
    Ok(())
}

/// For any input in any of the four modes, `erl_parse::Parser::finish`
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
        let mut p = erl_parse::Parser::new(mode);
        for t in &tokens {
            p.push_token(*t);
        }
        let _tree = p.finish();
        touched.set();
        Ok(())
    })?;
    assert!(
        touched.hit(),
        "no case exercised parser termination\n{runner}"
    );
    Ok(())
}
