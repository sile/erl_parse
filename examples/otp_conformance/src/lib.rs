//! Shared tokenize / preprocess / parse driver for OTP conformance binaries.
//!
//! Skip lists and include-path assembly follow `erl_pp`'s OTP smoke example.
#![warn(missing_docs)]
#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod predef;

pub use predef::{otp_release_from_env, otp_release_from_tag};

/// Outcome of one pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// The stage succeeded.
    Ok,
    /// The stage failed.
    Err,
}

impl Stage {
    /// Returns `"ok"` or `"err"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Err => "err",
        }
    }

    /// Parses `"ok"` / `"err"`.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ok" => Some(Self::Ok),
            "err" => Some(Self::Err),
            _ => None,
        }
    }
}

/// Result of driving tokenize / preprocess / parse on one input.
pub struct ParseRun {
    /// Tokenize stage.
    pub tokenize: Stage,
    /// Preprocess stage (`erl_pp`).
    pub preprocess: Stage,
    /// Parse stage. [`Stage::Err`] if an earlier stage failed.
    pub parse: Stage,
    /// Finished tree. `None` when tokenize failed.
    pub tree: Option<erl_parse::SyntaxTree>,
    /// Top-level units returned by `next_node`.
    pub roots: Vec<erl_parse::NodeId>,
    /// Lexical token count (hidden tokens excluded).
    pub token_count: usize,
    /// `Event::Diagnostic` warnings.
    pub preprocess_warnings: usize,
    /// `Event::Diagnostic` errors.
    pub preprocess_diagnostics_error: usize,
    /// Tokenize failure message.
    pub tokenize_reason: Option<String>,
    /// Preprocess failure message.
    pub preprocess_reason: Option<String>,
    /// Original input text (used with `Token::text` when there are no includes).
    pub source: String,
    /// Source buffer for each parser token index (main file, includes, expansions).
    pub token_sources: Vec<Arc<erl_pp::Source>>,
}

/// Tokenize, preprocess with `erl_pp`, then parse.
///
/// `otp_release` is the integer `?OTP_RELEASE` expands to (from `OTP_TAG`).
pub fn parse_text(
    mode: erl_parse::ParseMode,
    display: &str,
    text: String,
    include_paths: &[PathBuf],
    erl_libs: &[PathBuf],
    otp_release: Option<u32>,
) -> ParseRun {
    let source = match erl_tokenize::scan_tokens(&text) {
        Ok(tokens) => erl_pp::Source::new(display, text.clone(), tokens),
        Err(e) => {
            return ParseRun {
                tokenize: Stage::Err,
                preprocess: Stage::Err,
                parse: Stage::Err,
                tree: None,
                roots: Vec::new(),
                token_count: 0,
                preprocess_warnings: 0,
                preprocess_diagnostics_error: 0,
                tokenize_reason: Some(format!("scan_tokens: {e}")),
                preprocess_reason: None,
                source: text,
                token_sources: Vec::new(),
            };
        }
    };
    let mut pp = erl_pp::Preprocessor::new([source]);
    let mut parser = erl_parse::Parser::new(mode);
    let mut predef = predef::PredefContext::new(otp_release);
    let mut roots = Vec::new();
    let mut token_sources = Vec::new();
    let mut token_count = 0usize;
    let mut warnings = 0usize;
    let mut diag_errors = 0usize;
    let mut preprocess_reason = None;

    loop {
        let event = pp
            .step()
            .expect("Preprocessor::step must not return ProtocolError");
        match event {
            erl_pp::Event::Token(t) => {
                token_count += 1;
                predef.on_token(&t);
                token_sources.push(Arc::clone(t.source()));
                parser.feed_token(*t.token());
                while let Some(id) = parser.next_node() {
                    roots.push(id);
                }
            }
            erl_pp::Event::MacroDefined(_) | erl_pp::Event::MacroUndefined(_) => {}
            erl_pp::Event::AwaitingInclude(req) => {
                let (included, load_failure) = resolve_include(&req, include_paths, erl_libs);
                if let Some(reason) = load_failure
                    && preprocess_reason.is_none()
                {
                    preprocess_reason = Some(reason);
                }
                pp.resume_include(included)
                    .expect("resume_include after AwaitingInclude");
            }
            erl_pp::Event::AwaitingConditional(req) => {
                let branch = match req {
                    erl_pp::Conditional::Ifdef(d) => match predef.ifdef_defined(d.name.as_str()) {
                        Some(true) => erl_pp::Branch::Then,
                        Some(false) => erl_pp::Branch::Else,
                        None => d.recommended,
                    },
                    erl_pp::Conditional::Ifndef(d) => match predef.ifdef_defined(d.name.as_str()) {
                        Some(true) => erl_pp::Branch::Else,
                        Some(false) => erl_pp::Branch::Then,
                        None => d.recommended,
                    },
                    erl_pp::Conditional::If(c) | erl_pp::Conditional::Elif(c) => predef
                        .if_branch(&c.condition_tokens)
                        .unwrap_or(erl_pp::Branch::Else),
                };
                pp.resume_conditional(branch)
                    .expect("resume_conditional after AwaitingConditional");
            }
            erl_pp::Event::AwaitingMacroExpansion(call) => {
                let source = match predef.expansion_text(&call) {
                    Ok(text) => match erl_tokenize::scan_tokens(&text) {
                        Ok(tokens) => erl_pp::Source::new("<predef>", text, tokens),
                        Err(e) => {
                            if preprocess_reason.is_none() {
                                preprocess_reason = Some(format!("predef scan_tokens: {e}"));
                            }
                            empty_source("<predef>")
                        }
                    },
                    Err(e) => {
                        if preprocess_reason.is_none() {
                            preprocess_reason = Some(e);
                        }
                        empty_source("<predef>")
                    }
                };
                pp.resume_macro_expansion(source)
                    .expect("resume_macro_expansion after AwaitingMacroExpansion");
            }
            erl_pp::Event::BranchBoundary(_) => {}
            erl_pp::Event::Diagnostic(d) => match d.severity {
                erl_pp::Severity::Error => diag_errors += 1,
                erl_pp::Severity::Warning => warnings += 1,
            },
            erl_pp::Event::PreprocessError(e) => {
                if preprocess_reason.is_none() {
                    preprocess_reason = Some(format!("preprocess: {e:?}"));
                }
            }
            erl_pp::Event::Complete => break,
        }
    }

    while let Some(id) = parser.next_node() {
        roots.push(id);
    }
    let tree = parser.finish();
    let preprocess = if preprocess_reason.is_none() {
        Stage::Ok
    } else {
        Stage::Err
    };
    let parse = if preprocess == Stage::Ok && accepted(&tree) {
        Stage::Ok
    } else {
        Stage::Err
    };
    ParseRun {
        tokenize: Stage::Ok,
        preprocess,
        parse,
        tree: Some(tree),
        roots,
        token_count,
        preprocess_warnings: warnings,
        preprocess_diagnostics_error: diag_errors,
        tokenize_reason: None,
        preprocess_reason,
        source: text,
        token_sources,
    }
}

/// True when the diagnostic list is empty and the tree has no `erl_parse::SyntaxKind::Error` node.
pub fn accepted(tree: &erl_parse::SyntaxTree) -> bool {
    tree.diagnostics().is_empty() && !has_error_node(tree)
}

fn has_error_node(tree: &erl_parse::SyntaxTree) -> bool {
    tree.roots().any(|root| {
        root.kind() == erl_parse::SyntaxKind::Error
            || root
                .descendants()
                .any(|n| n.kind() == erl_parse::SyntaxKind::Error)
    })
}

/// Maps a module-mode root to OTP's attribute / function / error classification.
pub fn form_category(kind: erl_parse::SyntaxKind) -> &'static str {
    match kind {
        erl_parse::SyntaxKind::Attribute => "attribute",
        erl_parse::SyntaxKind::FunctionDecl => "function",
        erl_parse::SyntaxKind::Error => "error",
        _ => "other",
    }
}

/// Classifies each root in `roots`.
pub fn form_categories(
    tree: &erl_parse::SyntaxTree,
    roots: &[erl_parse::NodeId],
) -> Vec<&'static str> {
    roots
        .iter()
        .filter_map(|id| tree.view(*id).map(|v| form_category(v.kind())))
        .collect()
}

/// True when `id` is a `-feature(...)` attribute consumed by OTP `epp` before `erl_parse`.
pub fn is_epp_consumed_feature_attribute(
    tree: &erl_parse::SyntaxTree,
    token_sources: &[Arc<erl_pp::Source>],
    id: erl_parse::NodeId,
) -> bool {
    let Some(view) = tree.view(id) else {
        return false;
    };
    if view.kind() != erl_parse::SyntaxKind::Attribute {
        return false;
    }
    let Some(name_node) = child_of_kind(view, erl_parse::SyntaxKind::AttributeName) else {
        return false;
    };
    for (idx, t) in name_node.tokens_in_range() {
        if t.kind().is_lexical()
            && let Some(text) = token_text(tree, token_sources, idx)
        {
            return text == "feature";
        }
    }
    false
}

fn child_of_kind(
    node: erl_parse::NodeView<'_>,
    kind: erl_parse::SyntaxKind,
) -> Option<erl_parse::NodeView<'_>> {
    node.children().find(|c| c.kind() == kind)
}

/// Roots aligned with OTP `erl_parse` per-form fixture lists (drops epp-consumed `-feature`).
pub fn roots_for_otp_parse_compare(
    tree: &erl_parse::SyntaxTree,
    token_sources: &[Arc<erl_pp::Source>],
    roots: &[erl_parse::NodeId],
) -> Vec<erl_parse::NodeId> {
    roots
        .iter()
        .copied()
        .filter(|id| !is_epp_consumed_feature_attribute(tree, token_sources, *id))
        .collect()
}

fn token_text<'a>(
    tree: &erl_parse::SyntaxTree,
    token_sources: &'a [Arc<erl_pp::Source>],
    index: erl_parse::TokenIndex,
) -> Option<&'a str> {
    let token = *tree.tokens().get(index.get())?;
    let source = token_sources.get(index.get())?;
    Some(token.text(source.text()))
}

/// 1-based line of a `Diagnostic` range start.
pub fn diagnostic_line(tree: &erl_parse::SyntaxTree, range: erl_parse::TokenRange) -> usize {
    let idx = range.start();
    if let Some(t) = tree.tokens().get(idx.get()).copied() {
        return t.start().line().get();
    }
    if idx.get() > 0
        && let Some(t) = tree.tokens().get(idx.get() - 1).copied()
    {
        return t.start().line().get();
    }
    1
}

/// Line of the first `Diagnostic`, if any.
pub fn first_diagnostic_line(tree: &erl_parse::SyntaxTree) -> Option<usize> {
    tree.diagnostics()
        .first()
        .map(|e| diagnostic_line(tree, e.range()))
}

fn empty_source(name: &str) -> erl_pp::Source {
    erl_pp::Source::new(name.to_string(), String::new(), Vec::new())
}

fn resolve_include(
    include: &erl_pp::IncludeDirective,
    include_paths: &[PathBuf],
    erl_libs: &[PathBuf],
) -> (erl_pp::Source, Option<String>) {
    let raw_path = include.path.as_str();
    match erl_pp::open_include(include, include_paths, erl_libs) {
        Ok(path) => match fs::read_to_string(&path) {
            Ok(text) => match erl_tokenize::scan_tokens(&text) {
                Ok(tokens) => (
                    erl_pp::Source::new(path.to_string_lossy().into_owned(), text, tokens),
                    None,
                ),
                Err(e) => (
                    empty_source(raw_path),
                    Some(format!(
                        "include scan_tokens failed for {}: {e}",
                        path.display()
                    )),
                ),
            },
            Err(e) => (
                empty_source(raw_path),
                Some(format!("include read failed for {}: {e}", path.display())),
            ),
        },
        Err(e) => (
            empty_source(raw_path),
            Some(format!("open_include({raw_path}): {e}")),
        ),
    }
}

/// Builds the `open_include` search list. `extra` is CLI `-I`.
pub fn build_include_paths(
    target: &Path,
    root: &Path,
    globals: &[PathBuf],
    extra: &[PathBuf],
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for e in extra {
        if !paths.iter().any(|p| p == e) {
            paths.push(e.clone());
        }
    }
    if let Some(dir) = target.parent() {
        paths.push(dir.to_path_buf());
        if let Some(parent) = dir.parent() {
            let include = parent.join("include");
            if include != *dir {
                paths.push(include);
            }
        }
    }
    if let Some(app_src) = find_app_src(target) {
        for sub in walk_dirs(&app_src) {
            if !paths.iter().any(|p| p == &sub) {
                paths.push(sub);
            }
        }
    }
    if target.to_string_lossy().contains("/erts/preloaded/src/") {
        let kernel_src = root.join("lib").join("kernel").join("src");
        if kernel_src.is_dir() && !paths.iter().any(|p| p == &kernel_src) {
            paths.push(kernel_src);
        }
    }
    for g in globals {
        if !paths.iter().any(|p| p == g) {
            paths.push(g.clone());
        }
    }
    paths
}

fn find_app_src(target: &Path) -> Option<PathBuf> {
    let mut cur = target.parent()?;
    loop {
        if cur.file_name().and_then(|n| n.to_str()) == Some("src") {
            return Some(cur.to_path_buf());
        }
        cur = cur.parent()?;
    }
}

fn walk_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if root.is_dir() {
        out.push(root.to_path_buf());
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    out.extend(walk_dirs(&p));
                }
            }
        }
    }
    out
}

/// Collects `otp/lib/<app>/include/` for every application.
pub fn collect_app_include_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let lib = root.join("lib");
    if let Ok(entries) = fs::read_dir(&lib) {
        for entry in entries.flatten() {
            let include = entry.path().join("include");
            if include.is_dir() {
                out.push(include);
            }
        }
    }
    out.sort();
    out
}

/// Recursively collects `.erl` / `.hrl` files.
pub fn collect_erl_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_erl_files(&p, out);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str())
                && (ext == "erl" || ext == "hrl")
            {
                out.push(p);
            }
        }
    }
}

/// OTP application source plus preloaded ERTS modules.
pub fn is_target(path: &Path) -> bool {
    let s = path.to_string_lossy();
    let in_lib_src_or_include = s.contains("/lib/")
        && (s.contains("/src/") || s.contains("/include/"))
        && !s.contains("/test/");
    let in_erts_preloaded = s.contains("/erts/preloaded/src/");
    in_lib_src_or_include || in_erts_preloaded
}

/// Files that cannot be tokenized or preprocessed from a plain OTP checkout.
pub fn is_skipped(path: &Path) -> bool {
    let s = path.to_string_lossy();
    // wxWidgets binding templates embed `~s,` placeholders, not Erlang.
    if s.contains("/lib/wx/api_gen/wx_extra/") {
        return true;
    }
    // Literal NUL bytes; `erl_scan` also rejects this file.
    if s.ends_with("/lib/kernel/test/socket_sctp_SUITE.erl") {
        return true;
    }
    // Need headers produced by the OTP build (codegen / MIB / asn1).
    if s.contains("/lib/wx/")
        || s.contains("/lib/snmp/")
        || s.contains("/lib/public_key/src/")
        || s.contains("/lib/eldap/src/")
    {
        return true;
    }
    const INDIVIDUAL_SKIPS: &[&str] = &[
        "/lib/compiler/src/beam_ssa_alias.erl",       // -if / -elif
        "/lib/compiler/src/beam_ssa_alias_debug.hrl", // -if / -elif
        "/lib/compiler/src/beam_ssa_ss.erl",          // -if / -elif
        "/lib/stdlib/src/graph.erl",                  // -if / -elif
        "/lib/stdlib/src/peer.erl",                   // -if / -elif
        "/lib/compiler/src/beam_asm.erl",             // generated beam_opcodes.hrl
        "/lib/compiler/src/beam_disasm.erl",          // generated beam_opcodes.hrl
        "/lib/kernel/src/inet_dns.erl",               // generated inet_dns_record_adts.hrl
        "/lib/common_test/src/ct_snmp.erl",           // snmp_types.hrl (snmp app is skipped)
        "/lib/stdlib/src/io_ansi.erl",                // call as binary pattern size
        "/lib/syntax_tools/src/merl_tests.erl",       // merl parse-transform quotes in patterns
    ];
    INDIVIDUAL_SKIPS.iter().any(|suffix| s.ends_with(suffix))
}

/// Number of non-`Error` roots after the first `erl_parse::SyntaxKind::Error` root.
pub fn later_forms_after_error(tree: &erl_parse::SyntaxTree, roots: &[erl_parse::NodeId]) -> usize {
    let mut seen_error = false;
    let mut later = 0usize;
    for id in roots {
        let Some(view) = tree.view(*id) else {
            continue;
        };
        if view.kind() == erl_parse::SyntaxKind::Error {
            seen_error = true;
            continue;
        }
        if seen_error {
            later += 1;
        }
    }
    later
}

/// Nested JSON array used to compare operator precedence (not pretty-print).
pub fn tree_shape(
    tree: &erl_parse::SyntaxTree,
    source: &str,
    id: erl_parse::NodeId,
) -> Option<String> {
    let view = tree.view(id)?;
    Some(shape_node(tree, source, view))
}

fn unwrap_paren(node: erl_parse::NodeView<'_>) -> erl_parse::NodeView<'_> {
    let mut cur = node;
    while cur.kind() == erl_parse::SyntaxKind::ParenExpr
        && let Some(child) = cur.children().next()
    {
        cur = child;
    }
    cur
}

fn shape_node(tree: &erl_parse::SyntaxTree, source: &str, node: erl_parse::NodeView<'_>) -> String {
    let node = unwrap_paren(node);
    match node.kind() {
        erl_parse::SyntaxKind::BinaryOpExpr
        | erl_parse::SyntaxKind::MatchExpr
        | erl_parse::SyntaxKind::SendExpr
        | erl_parse::SyntaxKind::MaybeMatchExpr => {
            let mut kids = node.children();
            let Some(left) = kids.next() else {
                return "[]".to_string();
            };
            let Some(right) = kids.next() else {
                return "[]".to_string();
            };
            let op = operator_between(tree, source, left.range().end(), right.range().start())
                .unwrap_or_else(|| "_".to_string());
            format!(
                "[\"binop\",\"{}\",{},{}]",
                json_escape(&op),
                shape_node(tree, source, left),
                shape_node(tree, source, right)
            )
        }
        erl_parse::SyntaxKind::UnaryOpExpr | erl_parse::SyntaxKind::CatchExpr => {
            let Some(child) = node.children().next() else {
                return "[]".to_string();
            };
            let op = operator_between(tree, source, node.range().start(), child.range().start())
                .unwrap_or_else(|| "_".to_string());
            format!(
                "[\"unary\",\"{}\",{}]",
                json_escape(&op),
                shape_node(tree, source, child)
            )
        }
        erl_parse::SyntaxKind::IntegerExpr => "[\"integer\"]".to_string(),
        erl_parse::SyntaxKind::AtomExpr => "[\"atom\"]".to_string(),
        erl_parse::SyntaxKind::VarExpr => "[\"var\"]".to_string(),
        erl_parse::SyntaxKind::FloatExpr => "[\"float\"]".to_string(),
        erl_parse::SyntaxKind::CharExpr => "[\"char\"]".to_string(),
        other => format!("[\"{other:?}\"]"),
    }
}

fn operator_between(
    tree: &erl_parse::SyntaxTree,
    source: &str,
    start: erl_parse::TokenIndex,
    end: erl_parse::TokenIndex,
) -> Option<String> {
    if start.get() >= end.get() {
        return None;
    }
    let mut ops = Vec::new();
    for t in &tree.tokens()[erl_parse::TokenRange::new(start, end).as_range()] {
        if t.kind().is_lexical() {
            ops.push(t.text(source).to_string());
        }
    }
    if ops.is_empty() {
        None
    } else {
        Some(ops.join(" "))
    }
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

/// Intentional OTP divergences; matching `id` is excluded from ERROR.
pub fn named_divergence(id: &str) -> Option<&'static str> {
    const TABLE: &[(&str, &str)] = &[
        (
            "syn:expr-mode:comma-seq",
            "expression mode top-level does not accept expression lists",
        ),
        (
            "syn:err:missing-dot",
            "finish flushes an unterminated last unit; OTP parse_form reports an error",
        ),
    ];
    TABLE
        .iter()
        .find(|(prefix, _)| id == *prefix)
        .map(|(_, reason)| *reason)
}

/// Parses a fixture `mode` string.
pub fn parse_mode_from_str(s: &str) -> Option<erl_parse::ParseMode> {
    match s {
        "module" => Some(erl_parse::ParseMode::Module),
        "term_list" => Some(erl_parse::ParseMode::TermList),
        "expression" => Some(erl_parse::ParseMode::Expression),
        "type" => Some(erl_parse::ParseMode::Type),
        _ => None,
    }
}

/// OTP checkout tag from `OTP_TAG`. Empty or unset is `None`.
pub fn otp_tag_from_env() -> Option<String> {
    match std::env::var("OTP_TAG") {
        Ok(s) if !s.is_empty() => Some(s),
        _ => None,
    }
}

/// Infers the OTP checkout root from a path recorded in the fixture.
pub fn otp_root_from_path(path: &Path) -> Option<PathBuf> {
    let s = path.to_string_lossy();
    for marker in ["/lib/", "/erts/"] {
        if let Some(i) = s.find(marker) {
            let root = &s[..i];
            if root.is_empty() {
                return Some(PathBuf::from("."));
            }
            return Some(PathBuf::from(root));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_for_otp_parse_compare_drops_feature_attribute() {
        let src = "-module(m).\n-feature(maybe_expr, enable).\n-export([f/0]).\n";
        let run = parse_text(
            erl_parse::ParseMode::Module,
            "t",
            src.to_string(),
            &[],
            &[] as &[PathBuf],
            Some(29),
        );
        let tree = run.tree.as_ref().expect("parse_text succeeded");
        assert_eq!(run.roots.len(), 3);
        assert!(is_epp_consumed_feature_attribute(
            tree,
            &run.token_sources,
            run.roots[1]
        ));
        let cmp = roots_for_otp_parse_compare(tree, &run.token_sources, &run.roots);
        assert_eq!(cmp.len(), 2);
        assert_eq!(cmp[0], run.roots[0]);
        assert_eq!(cmp[1], run.roots[2]);
    }
}
