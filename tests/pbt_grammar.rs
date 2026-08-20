//! Property-based tests that exercise the parser at the mode /
//! entry-point level: determinism, observation-invariance, and the
//! `ProtocolError` contract on auxiliary entry points.

#[expect(dead_code, reason = "shared harness; this binary uses only a subset")]
mod pbt_harness;

use erl_parse::{ParseMode, Parser, TokenIndex, TokenRange};
use pbt_harness::{
    CASES, Counter, Flag, LabelSet, SEED_ENV, sample_expression, sample_expression_unit,
    sample_module_source, sample_term, sample_term_list_source, scan_all,
};

/// Two fresh `Parser` instances driven with the same tokens in the
/// same order produce byte-identical `SyntaxIndex` entries and
/// `ParseError` sequences.
#[test]
fn determinism_across_two_parsers() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let mut runner = noprop::Runner::new(seed);
    let modes = &[
        ParseMode::Expression,
        ParseMode::Module,
        ParseMode::TermList,
    ];
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
        let mut a = Parser::new(mode);
        let mut b = Parser::new(mode);
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
/// `SyntaxIndex` / `ParseError` regardless of whether the caller
/// interleaves `next_top_node` / `state` / `syntax_tree`
/// observations between `push_token` calls or defers all
/// observation until `finish` returns.
#[test]
fn observation_invariance() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let saw_observation = Counter::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(CASES, |ctx| {
        let src = sample_module_source(ctx);
        let Some(tokens) = scan_all(&src) else {
            return Ok(());
        };
        let mut quiet = Parser::new(ParseMode::Module);
        let mut noisy = Parser::new(ParseMode::Module);
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

/// The five auxiliary entry points return `Ok(_)` when called on
/// a fresh parser instance whose only pushed tokens were the range
/// they operate on. `ProtocolError` fires only when a top-level
/// unit is in progress, which the drain protocol below prevents.
#[test]
fn aux_entry_points_respect_protocol_when_freshly_used() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let saw_each = LabelSet::new();
    let mut runner = noprop::Runner::new(seed);
    runner.run(CASES, |ctx| {
        // Draw one of the five aux entry points and a matching
        // source fragment. Use Expression mode so no top-level
        // dot-driver runs while we push tokens (Expression mode
        // triggers only on lexical dots; we omit dots).
        let variant = noprop::sample_weighted_index(ctx, &[1, 1, 1, 1, 1]);
        let src = match variant {
            0 => sample_expression(ctx, 2),
            1 => sample_expression(ctx, 2),
            2 => "X > 0, X < 10".to_string(),
            3 => sample_term(ctx, 2),
            _ => "integer() | atom()".to_string(),
        };
        let Some(tokens) = scan_all(&src) else {
            return Ok(());
        };
        if tokens.is_empty() {
            return Ok(());
        }
        let mut p = Parser::new(ParseMode::Expression);
        for t in &tokens {
            p.push_token(*t);
        }
        let end = p.syntax_tree().tokens().end_index();
        let range = TokenRange::new(TokenIndex::new(0), end);
        let result = match variant {
            0 => p.parse_expression_range(range),
            1 => p.parse_pattern_range(range),
            2 => p.parse_guard_range(range),
            3 => p.parse_term_range(range),
            _ => p.parse_type_range(range),
        };
        assert!(
            result.is_ok(),
            "aux entry point {variant} returned ProtocolError on fresh parser: source {src:?}"
        );
        let _ = p.finish();
        saw_each.insert(match variant {
            0 => "expression",
            1 => "pattern",
            2 => "guard",
            3 => "term",
            _ => "type",
        });
        Ok(())
    })?;
    for label in ["expression", "pattern", "guard", "term", "type"] {
        assert!(
            saw_each.contains(label),
            "aux entry point {label} was never exercised\n{runner}"
        );
    }
    Ok(())
}

/// For any input in any of the three modes, `Parser::finish`
/// returns without panicking or hanging: parsing terminates.
#[test]
fn parser_always_terminates_across_modes() -> noprop::TestResult {
    let seed = noprop::seed_from_env_or_time(SEED_ENV)?;
    let modes = &[
        ParseMode::Expression,
        ParseMode::Module,
        ParseMode::TermList,
    ];
    let touched = Flag::new();
    let mut runner = noprop::Runner::new(seed);
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
        let mut p = Parser::new(mode);
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
