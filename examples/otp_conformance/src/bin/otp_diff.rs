//! Compare this crate's parser against an OTP `erl_parse` JSONL fixture.
//!
//! The OTP tag is not baked in. `otp_diff` reads `OTP_TAG` and warns if it
//! disagrees with the fixture `_meta.otp_tag` (written by the dump escript
//! from the same variable).
//!
//! ```text
//! OTP_TAG=... cargo run -p otp_conformance --release --bin otp_diff -- <fixture.jsonl>
//! ```
//!
//! Tokenize / preprocess XOR is WARN (exit 0). Parse XOR is ERROR (exit 1).

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use erl_parse::ParseMode;
use nojson::{RawJson, RawJsonValue};
use otp_conformance::{
    AuxKind, Stage, build_include_paths, collect_app_include_dirs, first_error_line,
    form_categories, later_forms_after_error, named_divergence, otp_root_from_path,
    otp_tag_from_env, parse_aux, parse_mode_from_str, parse_text, tree_shape,
};

fn main() -> noargs::Result<ExitCode> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = "otp_diff";
    args.metadata_mut().app_description =
        "Compare parser output against an OTP erl_parse JSONL fixture";
    noargs::HELP_FLAG.take_help(&mut args);

    let fixture_path: PathBuf = noargs::arg("<fixture.jsonl>")
        .doc("JSONL fixture produced by scripts/dump-otp-parse-fixture.escript")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(ExitCode::SUCCESS);
    }

    let fixture = fs::read_to_string(&fixture_path).map_err(|e| e.to_string())?;

    let mut records = 0usize;
    let mut ok = 0usize;
    let mut warn = 0usize;
    let mut error = 0usize;
    let mut skipped = 0usize;

    for (lineno, line) in fixture.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let raw = match RawJson::parse(line) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("{}:{}: json parse: {e}", fixture_path.display(), lineno + 1);
                error += 1;
                continue;
            }
        };
        let value = raw.value();
        if value
            .to_member("_meta")
            .ok()
            .and_then(|m| m.optional())
            .is_some()
        {
            if let Err(msg) = check_meta(value) {
                eprintln!("WARN fixture metadata: {msg}");
                warn += 1;
            }
            continue;
        }
        records += 1;
        match compare_record(value) {
            Compare::Ok => ok += 1,
            Compare::Warn(msg) => {
                warn += 1;
                eprintln!("WARN {msg}");
            }
            Compare::Error(msg) => {
                error += 1;
                eprintln!("ERROR {msg}");
            }
            Compare::Skip => skipped += 1,
        }
    }

    eprintln!("RECORDS: {records}, OK: {ok}, WARN: {warn}, ERROR: {error}, SKIP: {skipped}");
    Ok(ExitCode::from(u8::from(error > 0)))
}

enum Compare {
    Ok,
    Warn(String),
    Error(String),
    Skip,
}

struct FormRec {
    parse: Stage,
    category: String,
    line: Option<usize>,
}

fn check_meta(v: RawJsonValue<'_, '_>) -> Result<(), String> {
    let fixture_tag = req_str(v, "otp_tag")?;
    match otp_tag_from_env() {
        None => Err("OTP_TAG is unset; not verifying fixture otp_tag".into()),
        Some(env_tag) if env_tag == fixture_tag => Ok(()),
        Some(env_tag) => Err(format!(
            "fixture otp_tag {fixture_tag} != OTP_TAG {env_tag}"
        )),
    }
}

fn compare_record(v: RawJsonValue<'_, '_>) -> Compare {
    let id = match req_str(v, "id") {
        Ok(s) => s,
        Err(e) => return Compare::Error(e),
    };
    if let Some(reason) = named_divergence(&id) {
        eprintln!("SKIP {id}: {reason}");
        return Compare::Skip;
    }
    let kind = match req_str(v, "kind") {
        Ok(s) => s,
        Err(e) => return Compare::Error(format!("{id}: {e}")),
    };
    let mode = match req_str(v, "mode")
        .and_then(|s| parse_mode_from_str(&s).ok_or_else(|| format!("unknown mode {s}")))
    {
        Ok(m) => m,
        Err(e) => return Compare::Error(format!("{id}: {e}")),
    };
    let exp_tok = match req_stage(v, "tokenize") {
        Ok(s) => s,
        Err(e) => return Compare::Error(format!("{id}: {e}")),
    };
    let exp_pp = match req_stage(v, "preprocess") {
        Ok(s) => s,
        Err(e) => return Compare::Error(format!("{id}: {e}")),
    };

    let (text, display, include_paths, erl_libs) = match load_input(v, &kind) {
        Ok(t) => t,
        Err(e) => return Compare::Error(format!("{id}: {e}")),
    };

    let run = parse_text(mode, &display, text, &include_paths, &erl_libs);

    if let Some(c) = xor_stage(&id, "tokenize", exp_tok, run.tokenize) {
        return c;
    }
    if exp_tok == Stage::Err {
        return Compare::Skip;
    }
    if let Some(c) = xor_stage(&id, "preprocess", exp_pp, run.preprocess) {
        return c;
    }
    if exp_pp == Stage::Err {
        return Compare::Skip;
    }

    compare_parse(&id, v, &kind, mode, &run)
}

fn load_input(
    v: RawJsonValue<'_, '_>,
    kind: &str,
) -> Result<(String, String, Vec<PathBuf>, Vec<PathBuf>), String> {
    if kind == "otp_file" {
        let path = PathBuf::from(req_str(v, "path")?);
        let text =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let display = path.to_string_lossy().into_owned();
        let root = otp_root_from_path(&path)
            .ok_or_else(|| format!("cannot infer OTP root from {}", path.display()))?;
        let globals = collect_app_include_dirs(&root);
        let include_paths = build_include_paths(&path, &root, &globals, &[]);
        let erl_libs = vec![root.join("lib")];
        Ok((text, display, include_paths, erl_libs))
    } else {
        let text = req_str(v, "source")?;
        Ok((text, "<synthetic>".to_string(), Vec::new(), Vec::new()))
    }
}

fn xor_stage(id: &str, stage: &str, expected: Stage, actual: Stage) -> Option<Compare> {
    if expected == actual {
        None
    } else {
        Some(Compare::Warn(format!(
            "{id}: {stage} XOR (otp {}, rust {})",
            expected.as_str(),
            actual.as_str()
        )))
    }
}

fn compare_parse(
    id: &str,
    v: RawJsonValue<'_, '_>,
    kind: &str,
    mode: ParseMode,
    run: &otp_conformance::ParseRun,
) -> Compare {
    let Some(tree) = run.tree.as_ref() else {
        return Compare::Error(format!(
            "{id}: missing syntax tree after successful preprocess"
        ));
    };

    if kind == "recovery" {
        return compare_recovery(id, v, tree, &run.roots);
    }

    let forms = match parse_forms(v) {
        Ok(f) => f,
        Err(e) => return Compare::Error(format!("{id}: {e}")),
    };
    if forms.is_empty() {
        let exp = match opt_stage(v, "parse") {
            Ok(Some(s)) => s,
            Ok(None) => {
                return Compare::Error(format!("{id}: missing parse or forms"));
            }
            Err(e) => return Compare::Error(format!("{id}: {e}")),
        };
        return compare_one_parse(id, v, mode, run, tree, exp);
    }

    if run.parse == Stage::Ok && forms.iter().any(|f| f.parse == Stage::Err) {
        return Compare::Error(format!(
            "{id}: parse XOR (otp has a rejected form, rust accepted the file)"
        ));
    }
    if run.parse == Stage::Err && forms.iter().all(|f| f.parse == Stage::Ok) {
        return Compare::Error(format!(
            "{id}: parse XOR (otp accepted all forms, rust rejected the file)"
        ));
    }
    if run.parse == Stage::Err {
        if let Some(exp_line) = forms
            .iter()
            .find_map(|f| if f.parse == Stage::Err { f.line } else { None })
            && let Some(act_line) = first_error_line(tree)
            && act_line != exp_line
        {
            return Compare::Error(format!("{id}: error line otp {exp_line} rust {act_line}"));
        }
        return Compare::Ok;
    }

    if run.roots.len() != forms.len() {
        return Compare::Error(format!(
            "{id}: form count otp {} rust {}",
            forms.len(),
            run.roots.len()
        ));
    }
    if mode == ParseMode::Module {
        let cats = form_categories(tree, &run.roots);
        for (i, (got, exp)) in cats.iter().zip(forms.iter()).enumerate() {
            if *got != exp.category.as_str() {
                return Compare::Error(format!(
                    "{id}: form[{i}] category otp {} rust {got}",
                    exp.category
                ));
            }
        }
    }
    compare_aux_and_tree(id, v, mode, run, tree)
}

fn compare_one_parse(
    id: &str,
    v: RawJsonValue<'_, '_>,
    mode: ParseMode,
    run: &otp_conformance::ParseRun,
    tree: &erl_parse::SyntaxTree,
    exp: Stage,
) -> Compare {
    if exp != run.parse {
        return Compare::Error(format!(
            "{id}: parse XOR (otp {}, rust {})",
            exp.as_str(),
            run.parse.as_str()
        ));
    }
    if exp == Stage::Err {
        match (opt_usize(v, "error_line"), first_error_line(tree)) {
            (Ok(Some(exp_line)), Some(act_line)) if exp_line != act_line => {
                return Compare::Error(format!("{id}: error line otp {exp_line} rust {act_line}"));
            }
            (Err(e), _) => return Compare::Error(format!("{id}: {e}")),
            _ => return Compare::Ok,
        }
    }
    compare_aux_and_tree(id, v, mode, run, tree)
}

fn compare_aux_and_tree(
    id: &str,
    v: RawJsonValue<'_, '_>,
    mode: ParseMode,
    run: &otp_conformance::ParseRun,
    tree: &erl_parse::SyntaxTree,
) -> Compare {
    if let Some(expected_tree) = match opt_tree(v) {
        Ok(t) => t,
        Err(e) => return Compare::Error(format!("{id}: {e}")),
    } {
        let Some(root) = run.roots.first().copied() else {
            return Compare::Error(format!("{id}: no root for tree comparison"));
        };
        let Some(actual) = tree_shape(tree, &run.source, root) else {
            return Compare::Error(format!("{id}: failed to reconstruct tree shape"));
        };
        if actual != expected_tree {
            return Compare::Error(format!("{id}: tree otp {expected_tree} rust {actual}"));
        }
    }

    let aux_kind = match opt_str(v, "aux_kind") {
        Ok(s) => s,
        Err(e) => return Compare::Error(format!("{id}: {e}")),
    };
    if let Some(kind_s) = aux_kind {
        let Some(aux_kind) = AuxKind::parse(&kind_s) else {
            return Compare::Error(format!("{id}: unknown aux_kind {kind_s}"));
        };
        let path = match req_str(v, "extract_path") {
            Ok(p) => p,
            Err(e) => return Compare::Error(format!("{id}: {e}")),
        };
        let exp = match opt_stage(v, "aux_parse") {
            Ok(Some(s)) => s,
            Ok(None) => Stage::Ok,
            Err(e) => return Compare::Error(format!("{id}: {e}")),
        };
        let ok = match parse_aux(mode, tree.tokens().as_slice(), aux_kind, &path) {
            Ok(b) => b,
            Err(e) => return Compare::Error(format!("{id}: aux {e}")),
        };
        let actual = if ok { Stage::Ok } else { Stage::Err };
        if actual != exp {
            return Compare::Error(format!(
                "{id}: aux_parse XOR (otp {}, rust {})",
                exp.as_str(),
                actual.as_str()
            ));
        }
    }
    Compare::Ok
}

fn compare_recovery(
    id: &str,
    v: RawJsonValue<'_, '_>,
    tree: &erl_parse::SyntaxTree,
    roots: &[erl_parse::NodeId],
) -> Compare {
    let expect = match opt_usize(v, "expect_later_forms") {
        Ok(Some(n)) => n,
        Ok(None) => {
            return Compare::Error(format!("{id}: recovery record needs expect_later_forms"));
        }
        Err(e) => return Compare::Error(format!("{id}: {e}")),
    };
    let got = later_forms_after_error(tree, roots);
    if got < expect {
        Compare::Error(format!(
            "{id}: recovery later forms {got} < expected {expect}"
        ))
    } else {
        Compare::Ok
    }
}

fn parse_forms(v: RawJsonValue<'_, '_>) -> Result<Vec<FormRec>, String> {
    let Some(arr) = v.to_member("forms").map_err(|e| e.to_string())?.optional() else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for item in arr.to_array().map_err(|e| e.to_string())? {
        let parse = req_stage(item, "parse")?;
        let category = opt_str(item, "category")?.unwrap_or_else(|| "other".to_string());
        let line = opt_usize(item, "line")?;
        out.push(FormRec {
            parse,
            category,
            line,
        });
    }
    Ok(out)
}

fn opt_tree(v: RawJsonValue<'_, '_>) -> Result<Option<String>, String> {
    match v.to_member("tree").map_err(|e| e.to_string())?.optional() {
        None => Ok(None),
        Some(t) => Ok(Some(compact_json(t)?)),
    }
}

fn compact_json(v: RawJsonValue<'_, '_>) -> Result<String, String> {
    if let Ok(arr) = v.to_array() {
        let mut parts = Vec::new();
        for item in arr {
            parts.push(compact_json(item)?);
        }
        return Ok(format!("[{}]", parts.join(",")));
    }
    if let Ok(s) = v.to_unquoted_string_str() {
        return Ok(format!("\"{}\"", json_escape(&s)));
    }
    Err("tree node must be a JSON array or string".into())
}

fn json_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out
}

fn req_str(v: RawJsonValue<'_, '_>, name: &str) -> Result<String, String> {
    let m = v
        .to_member(name)
        .map_err(|e| e.to_string())?
        .required()
        .map_err(|e| e.to_string())?;
    Ok(m.to_unquoted_string_str()
        .map_err(|e| e.to_string())?
        .into_owned())
}

fn opt_str(v: RawJsonValue<'_, '_>, name: &str) -> Result<Option<String>, String> {
    match v.to_member(name).map_err(|e| e.to_string())?.optional() {
        None => Ok(None),
        Some(x) => Ok(Some(
            x.to_unquoted_string_str()
                .map_err(|e| e.to_string())?
                .into_owned(),
        )),
    }
}

fn req_stage(v: RawJsonValue<'_, '_>, name: &str) -> Result<Stage, String> {
    let s = req_str(v, name)?;
    Stage::parse(&s).ok_or_else(|| format!("{name} must be \"ok\" or \"err\""))
}

fn opt_stage(v: RawJsonValue<'_, '_>, name: &str) -> Result<Option<Stage>, String> {
    match opt_str(v, name)? {
        None => Ok(None),
        Some(s) => Stage::parse(&s)
            .map(Some)
            .ok_or_else(|| format!("{name} must be \"ok\" or \"err\"")),
    }
}

fn opt_usize(v: RawJsonValue<'_, '_>, name: &str) -> Result<Option<usize>, String> {
    match v.to_member(name).map_err(|e| e.to_string())?.optional() {
        None => Ok(None),
        Some(x) => {
            let n: u64 = x
                .try_into()
                .map_err(|e: nojson::JsonParseError| e.to_string())?;
            Ok(Some(n as usize))
        }
    }
}
