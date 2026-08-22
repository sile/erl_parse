//! Smoke-test tokenize → preprocess → parse over an OTP source tree.
//!
//! ```text
//! cargo run -p otp_conformance --release --bin check_otp_parse -- <OTP_ROOT> [-I <include_dir>]...
//! ```
//!
//! Preprocess-only failures are WARN and do not fail the process.
//! Any file that preprocesses cleanly but still has parse errors fails the process.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> noargs::Result<ExitCode> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = "check_otp_parse";
    args.metadata_mut().app_description =
        "Parse all target .erl/.hrl files under an OTP source root directory";
    noargs::HELP_FLAG.take_help(&mut args);

    let include_opt = noargs::opt("include")
        .short('I')
        .ty("DIR")
        .doc("Directory searched by open_include (repeatable)");
    let mut extra_includes = Vec::<PathBuf>::new();
    while let Some(dir) = include_opt
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?
    {
        extra_includes.push(dir);
    }

    let root: PathBuf = noargs::arg("<OTP_ROOT>")
        .doc("OTP source root directory (e.g. a checkout of erlang/otp)")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(ExitCode::SUCCESS);
    }

    let otp_release = otp_conformance::otp_release_from_env();

    let mut files = Vec::new();
    otp_conformance::collect_erl_files(&root, &mut files);
    files.retain(|p| otp_conformance::is_target(p) && !otp_conformance::is_skipped(p));
    files.sort();

    let erl_libs = vec![root.join("lib")];
    let global_include_dirs = otp_conformance::collect_app_include_dirs(&root);
    let mut tokenize_err_files = 0usize;
    let mut preprocess_err_files = 0usize;
    let mut parse_err_files = 0usize;
    let mut token_total = 0usize;
    let mut warning_total = 0usize;
    let mut diag_error_total = 0usize;
    let start = std::time::Instant::now();

    for path in &files {
        let display = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();
        let text = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("WARN {display}: read: {e}");
                tokenize_err_files += 1;
                continue;
            }
        };
        let include_paths = otp_conformance::build_include_paths(
            path,
            &root,
            &global_include_dirs,
            &extra_includes,
        );
        let run = otp_conformance::parse_text(
            erl_parse::ParseMode::Module,
            &display,
            text,
            &include_paths,
            &erl_libs,
            otp_release,
        );
        token_total += run.token_count;
        warning_total += run.preprocess_warnings;
        diag_error_total += run.preprocess_diagnostics_error;

        if run.tokenize == otp_conformance::Stage::Err {
            tokenize_err_files += 1;
            if let Some(reason) = &run.tokenize_reason {
                eprintln!("WARN {display}: tokenize: {reason}");
            }
            continue;
        }
        if run.preprocess == otp_conformance::Stage::Err {
            preprocess_err_files += 1;
            if let Some(reason) = &run.preprocess_reason {
                eprintln!("WARN {display}: {reason}");
            }
            continue;
        }
        if run.parse == otp_conformance::Stage::Err {
            parse_err_files += 1;
            let detail = run
                .tree
                .as_ref()
                .and_then(|tree| {
                    let err = tree.errors().first()?;
                    let line = otp_conformance::first_error_line(tree)?;
                    Some(format!(
                        "{:?} at line {line}, expected {:?}, found {:?}",
                        err.kind(),
                        err.expected(),
                        err.found().map(|t| t.kind())
                    ))
                })
                .unwrap_or_else(|| "parse errors".to_string());
            eprintln!("ERROR {display}: {detail}");
        }
    }

    println!(
        "FILES: {}\nTOKENIZE ERROR FILES: {}\nPREPROCESS ERROR FILES: {}\nPARSE ERROR FILES: {}\nTOTAL TOKENS: {}\nTOTAL PREPROCESS DIAGNOSTICS: {} warnings / {} errors\nELAPSED: {:?}",
        files.len(),
        tokenize_err_files,
        preprocess_err_files,
        parse_err_files,
        token_total,
        warning_total,
        diag_error_total,
        start.elapsed(),
    );
    let _ = std::io::stdout().flush();
    Ok(ExitCode::from(u8::from(parse_err_files > 0)))
}
