//! Caller-side OTP predefined macros for the conformance driver.
//!
//! `erl_pp` expands `?FILE` / `?LINE` itself. Everything else
//! (`?MODULE`, `?FUNCTION_NAME`, `?OTP_RELEASE`, …) arrives as
//! `AwaitingMacroExpansion`. This module tracks enough lexical form
//! structure to fill those in, without asking the parser to interpret
//! attribute names.

use std::collections::HashSet;

use erl_pp::SourceToken;
use erl_tokenize::{Keyword, Symbol, TokenKind, TokenValue};

/// OTP 29 `erl_features:all/0` names that `?FEATURE_AVAILABLE` reports.
const AVAILABLE_FEATURES: &[&str] = &["maybe_expr", "compr_assign"];

/// Feature names enabled by default for OTP 29 (`erl_features:approved/0`).
const DEFAULT_ENABLED_FEATURES: &[&str] = &["maybe_expr"];

/// Lexical environment used to expand OTP predefined macros.
pub struct PredefContext {
    otp_release: Option<u32>,
    module: Option<String>,
    function_name: Option<String>,
    function_arity: Option<usize>,
    enabled_features: HashSet<String>,
    scan: FormScan,
}

#[derive(Default)]
struct FormScan {
    nest: usize,
    at_form_start: bool,
    hash: HashState,
    after_fun: bool,
    after_fun_name: bool,
    attr: AttrScan,
    fun: FunScan,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum HashState {
    #[default]
    Off,
    SawHash,
    SawHashName,
    SawWildcard,
}

#[derive(Default)]
enum AttrScan {
    #[default]
    Off,
    AfterHyphen,
    ModuleWaitParen,
    ModuleWaitName,
    FeatureWaitParen,
    FeatureWaitName,
    FeatureWaitComma {
        name: String,
    },
    FeatureWaitAction {
        name: String,
    },
    Other,
}

#[derive(Default)]
enum FunScan {
    #[default]
    Off,
    WaitParen {
        name: String,
    },
    InArity {
        name: String,
        depth: usize,
        items: usize,
        nonempty: bool,
    },
    Body {
        name: String,
        arity: usize,
    },
}

impl PredefContext {
    /// Builds a context. `otp_release` is the integer `?OTP_RELEASE` expands to.
    pub fn new(otp_release: Option<u32>) -> Self {
        Self {
            otp_release,
            module: None,
            function_name: None,
            function_arity: None,
            enabled_features: DEFAULT_ENABLED_FEATURES
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            scan: FormScan {
                at_form_start: true,
                ..FormScan::default()
            },
        }
    }

    /// Feeds one lexical preprocessor token into the form scanner.
    pub fn on_token(&mut self, token: &SourceToken) {
        self.scan
            .feed(token, &mut self.module, &mut self.enabled_features);
        match &self.scan.fun {
            FunScan::WaitParen { name } | FunScan::InArity { name, .. } => {
                self.function_name = Some(name.clone());
                self.function_arity = None;
            }
            FunScan::Body { name, arity } => {
                self.function_name = Some(name.clone());
                self.function_arity = Some(*arity);
            }
            FunScan::Off => {
                self.function_name = None;
                self.function_arity = None;
            }
        }
    }

    /// `Some(defined)` when `name` is an OTP predefined; `None` leaves
    /// `-ifdef` / `-ifndef` to `erl_pp`'s table-based `recommended`.
    pub fn ifdef_defined(&self, name: &str) -> Option<bool> {
        match name {
            "FILE" | "LINE" | "MACHINE" | "OTP_RELEASE" | "FEATURE_AVAILABLE"
            | "FEATURE_ENABLED" => Some(true),
            "MODULE" | "MODULE_STRING" => Some(self.module.is_some()),
            "FUNCTION_NAME" | "FUNCTION_ARITY" => Some(self.function_name.is_some()),
            _ => None,
        }
    }

    /// Erlang source text to splice in, or an error if the call is unknown
    /// or used before its value exists.
    pub fn expansion_text(&self, call: &erl_pp::MacroCall) -> Result<String, String> {
        let name = call.name.as_str();
        match (name, call.arity) {
            ("MODULE", None) => self
                .module
                .as_deref()
                .map(emit_atom)
                .ok_or_else(|| "?MODULE used before -module".to_string()),
            ("MODULE_STRING", None) => self
                .module
                .as_deref()
                .map(emit_string)
                .ok_or_else(|| "?MODULE_STRING used before -module".to_string()),
            ("MACHINE", None) => Ok("BEAM".to_string()),
            ("OTP_RELEASE", None) => self
                .otp_release
                .map(|n| n.to_string())
                .ok_or_else(|| "?OTP_RELEASE needs OTP_TAG".to_string()),
            ("FUNCTION_NAME", None) => self
                .function_name
                .as_deref()
                .map(emit_atom)
                .ok_or_else(|| "?FUNCTION_NAME used outside a function".to_string()),
            ("FUNCTION_ARITY", None) => self
                .function_arity
                .map(|n| n.to_string())
                .ok_or_else(|| "?FUNCTION_ARITY used outside a function".to_string()),
            ("FEATURE_AVAILABLE", Some(1)) => {
                let feat = feature_arg(call)?;
                Ok(emit_bool(AVAILABLE_FEATURES.contains(&feat.as_str())))
            }
            ("FEATURE_ENABLED", Some(1)) => {
                let feat = feature_arg(call)?;
                Ok(emit_bool(self.enabled_features.contains(&feat)))
            }
            _ => Err(format!(
                "unknown macro ?{name}{}",
                match call.arity {
                    None => String::new(),
                    Some(n) => format!("/{n}"),
                }
            )),
        }
    }
}

fn feature_arg(call: &erl_pp::MacroCall) -> Result<String, String> {
    let tokens = call
        .arguments
        .first()
        .ok_or_else(|| "feature macro is missing its argument".to_string())?;
    for t in tokens {
        if !t.token().kind().is_lexical() {
            continue;
        }
        return match t.value() {
            TokenValue::Atom(a) => Ok(a.into_owned()),
            _ => Err("feature macro argument is not an atom".to_string()),
        };
    }
    Err("feature macro argument is empty".to_string())
}

impl FormScan {
    fn feed(
        &mut self,
        token: &SourceToken,
        module: &mut Option<String>,
        enabled: &mut HashSet<String>,
    ) {
        let kind = token.token().kind();
        if self.after_fun {
            self.after_fun = false;
            match kind {
                TokenKind::Symbol(Symbol::OpenParen) => {
                    self.open_group();
                    return;
                }
                TokenKind::Atom | TokenKind::Variable => {
                    self.after_fun_name = true;
                    return;
                }
                TokenKind::Symbol(Symbol::Colon) => {
                    self.after_fun_name = true;
                    return;
                }
                _ => {}
            }
        }
        if self.after_fun_name {
            self.after_fun_name = false;
            match kind {
                TokenKind::Symbol(Symbol::Slash) => return,
                TokenKind::Symbol(Symbol::OpenParen) => {
                    self.open_group();
                    return;
                }
                TokenKind::Symbol(Symbol::Colon) => {
                    self.after_fun_name = true;
                    return;
                }
                TokenKind::Atom | TokenKind::Variable => {
                    self.after_fun_name = true;
                    return;
                }
                _ => {}
            }
        }

        match kind {
            TokenKind::Symbol(Symbol::OpenParen)
            | TokenKind::Symbol(Symbol::OpenSquare)
            | TokenKind::Symbol(Symbol::OpenBrace)
            | TokenKind::Symbol(Symbol::DoubleLeftAngle) => {
                // The `(` that opens a function's argument list is not
                // itself an argument; counting it as nonempty makes
                // `f()` look like arity 1.
                let opens_fun_args =
                    matches!(&self.fun, FunScan::WaitParen { .. }) && self.nest == 0;
                self.open_group();
                if !opens_fun_args {
                    self.on_arity_token(true);
                }
                self.hash = HashState::Off;
            }
            TokenKind::Symbol(Symbol::CloseParen)
            | TokenKind::Symbol(Symbol::CloseSquare)
            | TokenKind::Symbol(Symbol::CloseBrace)
            | TokenKind::Symbol(Symbol::DoubleRightAngle) => {
                self.close_group();
                self.finish_arity_if_closed();
                self.hash = HashState::Off;
            }
            TokenKind::Symbol(Symbol::Comma) => {
                self.on_arity_comma();
                self.hash = HashState::Off;
                self.advance_attr_comma();
            }
            TokenKind::Symbol(Symbol::Hyphen) if self.at_form_start && self.nest == 0 => {
                self.at_form_start = false;
                self.attr = AttrScan::AfterHyphen;
                self.hash = HashState::Off;
            }
            TokenKind::Symbol(Symbol::Sharp) => {
                self.hash = HashState::SawHash;
                self.on_arity_token(true);
            }
            TokenKind::Symbol(Symbol::WildcardRecord) => {
                self.hash = HashState::SawWildcard;
                self.on_arity_token(true);
            }
            TokenKind::Symbol(Symbol::Dot) => {
                if self.is_record_field_dot() {
                    self.hash = HashState::Off;
                    self.on_arity_token(true);
                } else if self.nest == 0 {
                    self.end_form();
                } else {
                    self.hash = HashState::Off;
                }
            }
            TokenKind::Keyword(Keyword::Begin)
            | TokenKind::Keyword(Keyword::Case)
            | TokenKind::Keyword(Keyword::If)
            | TokenKind::Keyword(Keyword::Receive)
            | TokenKind::Keyword(Keyword::Try)
            | TokenKind::Keyword(Keyword::Maybe)
            | TokenKind::Keyword(Keyword::Cond) => {
                self.nest += 1;
                self.hash = HashState::Off;
                self.on_arity_token(true);
            }
            TokenKind::Keyword(Keyword::Fun) => {
                self.after_fun = true;
                self.hash = HashState::Off;
                self.on_arity_token(true);
            }
            TokenKind::Keyword(Keyword::End) => {
                self.nest = self.nest.saturating_sub(1);
                self.hash = HashState::Off;
            }
            TokenKind::Atom => {
                if let TokenValue::Atom(a) = token.value() {
                    self.on_atom(a.as_ref(), module, enabled);
                }
            }
            TokenKind::Variable => {
                if self.hash == HashState::SawHash {
                    self.hash = HashState::SawHashName;
                } else {
                    self.hash = HashState::Off;
                }
                self.on_arity_token(true);
            }
            _ => {
                self.hash = HashState::Off;
                self.on_arity_token(true);
            }
        }
    }

    fn is_record_field_dot(&self) -> bool {
        matches!(self.hash, HashState::SawHashName | HashState::SawWildcard)
    }

    fn open_group(&mut self) {
        match &mut self.fun {
            FunScan::WaitParen { name } if self.nest == 0 => {
                let name = std::mem::take(name);
                self.fun = FunScan::InArity {
                    name,
                    depth: self.nest + 1,
                    items: 0,
                    nonempty: false,
                };
            }
            _ => {}
        }
        match &mut self.attr {
            AttrScan::ModuleWaitParen => self.attr = AttrScan::ModuleWaitName,
            AttrScan::FeatureWaitParen => self.attr = AttrScan::FeatureWaitName,
            _ => {}
        }
        self.nest += 1;
        if matches!(
            self.hash,
            HashState::SawHash | HashState::SawHashName | HashState::SawWildcard
        ) {
            self.hash = HashState::Off;
        }
    }

    fn close_group(&mut self) {
        self.nest = self.nest.saturating_sub(1);
    }

    fn on_arity_token(&mut self, nonempty: bool) {
        if let FunScan::InArity { nonempty: flag, .. } = &mut self.fun
            && nonempty
        {
            *flag = true;
        }
    }

    fn on_arity_comma(&mut self) {
        if let FunScan::InArity {
            depth,
            items,
            nonempty,
            ..
        } = &mut self.fun
            && self.nest == *depth
        {
            if *nonempty {
                *items += 1;
            }
            *nonempty = false;
        }
    }

    fn finish_arity_if_closed(&mut self) {
        let FunScan::InArity {
            name,
            depth,
            items,
            nonempty,
        } = &self.fun
        else {
            return;
        };
        if self.nest != *depth - 1 {
            return;
        }
        let arity = if *nonempty { *items + 1 } else { *items };
        self.fun = FunScan::Body {
            name: name.clone(),
            arity,
        };
    }

    fn on_atom(&mut self, atom: &str, module: &mut Option<String>, enabled: &mut HashSet<String>) {
        if self.hash == HashState::SawHash {
            self.hash = HashState::SawHashName;
        } else {
            self.hash = HashState::Off;
        }

        if self.at_form_start && self.nest == 0 && matches!(self.attr, AttrScan::Off) {
            self.at_form_start = false;
            self.fun = FunScan::WaitParen {
                name: atom.to_string(),
            };
            return;
        }

        match &self.attr {
            AttrScan::AfterHyphen => {
                self.attr = match atom {
                    "module" => AttrScan::ModuleWaitParen,
                    "feature" => AttrScan::FeatureWaitParen,
                    _ => AttrScan::Other,
                };
                return;
            }
            AttrScan::ModuleWaitName => {
                *module = Some(atom.to_string());
                self.attr = AttrScan::Other;
                return;
            }
            AttrScan::FeatureWaitName => {
                self.attr = AttrScan::FeatureWaitComma {
                    name: atom.to_string(),
                };
                return;
            }
            AttrScan::FeatureWaitAction { name } => {
                match atom {
                    "enable" => {
                        enabled.insert(name.clone());
                    }
                    "disable" => {
                        enabled.remove(name);
                    }
                    _ => {}
                }
                self.attr = AttrScan::Other;
                return;
            }
            _ => {}
        }

        self.on_arity_token(true);
    }

    fn advance_attr_comma(&mut self) {
        if let AttrScan::FeatureWaitComma { name } = &self.attr {
            self.attr = AttrScan::FeatureWaitAction { name: name.clone() };
        }
    }

    fn end_form(&mut self) {
        self.nest = 0;
        self.at_form_start = true;
        self.hash = HashState::Off;
        self.after_fun = false;
        self.after_fun_name = false;
        self.attr = AttrScan::Off;
        self.fun = FunScan::Off;
    }
}

/// Major release integer from tags such as `OTP-29.0.5` or `29.0.5`.
pub fn otp_release_from_tag(tag: &str) -> Option<u32> {
    let rest = tag.strip_prefix("OTP-").unwrap_or(tag);
    rest.split('.').next()?.parse().ok()
}

/// `?OTP_RELEASE` from `OTP_TAG`, if that env var is a usable tag.
pub fn otp_release_from_env() -> Option<u32> {
    crate::otp_tag_from_env()
        .as_deref()
        .and_then(otp_release_from_tag)
}

/// Scans `text` into a preprocessor [`erl_pp::Source`].
pub fn scanned_source(display: &str, text: &str) -> Result<erl_pp::Source, String> {
    let tokens = crate::scan_source(text)?;
    Ok(erl_pp::Source::new(
        display.to_string(),
        text.to_string(),
        tokens,
    ))
}

fn emit_atom(name: &str) -> String {
    if is_bare_atom(name) {
        name.to_string()
    } else {
        let mut out = String::from("'");
        for c in name.chars() {
            match c {
                '\\' | '\'' => {
                    out.push('\\');
                    out.push(c);
                }
                _ => {
                    out.push(c);
                }
            }
        }
        out.push('\'');
        out
    }
}

fn emit_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' | '"' => {
                out.push('\\');
                out.push(c);
            }
            _ => {
                out.push(c);
            }
        }
    }
    out.push('"');
    out
}

fn emit_bool(v: bool) -> String {
    if v { "true" } else { "false" }.to_string()
}

fn is_bare_atom(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && Keyword::from_text(name).is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Stage, accepted, parse_text};
    use erl_parse::ParseMode;
    use std::path::PathBuf;

    fn parse_ok(src: &str) {
        let run = parse_text(
            ParseMode::Module,
            "t.erl",
            src.to_string(),
            &[],
            &[] as &[PathBuf],
            Some(29),
        );
        assert_eq!(run.tokenize, Stage::Ok, "{src}");
        assert_eq!(
            run.preprocess,
            Stage::Ok,
            "{src} {:?}",
            run.preprocess_reason
        );
        let tree = run.tree.as_ref().expect("tree");
        assert!(accepted(tree), "{src} errors={:?}", tree.errors());
    }

    #[test]
    fn otp_release_from_otp_tag() {
        assert_eq!(otp_release_from_tag("OTP-29.0.5"), Some(29));
        assert_eq!(otp_release_from_tag("29.0.5"), Some(29));
        assert_eq!(otp_release_from_tag(""), None);
    }

    #[test]
    fn module_macro_expands_after_module_attribute() {
        parse_ok("-module(demo).\n-export([f/0]).\nf() -> ?MODULE.\n");
    }

    #[test]
    fn function_macros_expand_in_clause_body() {
        parse_ok("-module(demo).\nf(A, B) -> {?FUNCTION_NAME, ?FUNCTION_ARITY}.\n");
        parse_ok("-module(demo).\nf() -> ?FUNCTION_ARITY.\n");
    }

    #[test]
    fn otp_release_and_machine_expand() {
        parse_ok("-module(demo).\nf() -> {?MACHINE, ?OTP_RELEASE}.\n");
    }

    #[test]
    fn feature_macros_use_otp29_defaults() {
        parse_ok(
            "-module(demo).\nf() -> {?FEATURE_AVAILABLE(maybe_expr), ?FEATURE_ENABLED(maybe_expr), ?FEATURE_ENABLED(compr_assign)}.\n",
        );
    }

    #[test]
    fn ifdef_otp_release_takes_the_defined_branch() {
        parse_ok("-module(demo).\n-ifdef(OTP_RELEASE).\nf() -> ok.\n-endif.\n");
    }

    #[test]
    fn unknown_macro_is_a_preprocess_error() {
        let run = parse_text(
            ParseMode::Module,
            "t.erl",
            "-module(demo).\nf() -> ?NOT_A_REAL_MACRO.\n".to_string(),
            &[],
            &[] as &[PathBuf],
            Some(29),
        );
        assert_eq!(run.preprocess, Stage::Err);
        assert!(
            run.preprocess_reason
                .as_deref()
                .is_some_and(|s| s.contains("NOT_A_REAL_MACRO")),
            "{:?}",
            run.preprocess_reason
        );
    }

    #[test]
    fn module_before_attribute_is_a_preprocess_error() {
        let run = parse_text(
            ParseMode::Module,
            "t.erl",
            "f() -> ?MODULE.\n".to_string(),
            &[],
            &[] as &[PathBuf],
            Some(29),
        );
        assert_eq!(run.preprocess, Stage::Err);
    }

    #[test]
    fn logger_include_macros_expand() {
        let otp = PathBuf::from("/Users/tohta/dev/erlang/otp");
        let include = otp.join("lib/kernel/include");
        if !include.join("logger.hrl").is_file() {
            return;
        }
        let run = parse_text(
            ParseMode::Module,
            "t.erl",
            "-module(demo).\n-include(\"logger.hrl\").\nf() -> ?LOG_INFO(\"x\").\n".to_string(),
            &[include],
            &[otp.join("lib")],
            Some(29),
        );
        assert_eq!(run.preprocess, Stage::Ok, "{:?}", run.preprocess_reason);
        let tree = run.tree.as_ref().expect("tree");
        assert!(
            accepted(tree),
            "errors={:?} first={:?}",
            tree.errors(),
            tree.errors()
                .first()
                .map(|e| (e.kind(), e.expected(), e.found().map(|t| t.kind())))
        );
    }

    #[test]
    fn nested_predef_in_define_body_expands_at_use_site() {
        parse_ok(
            "-module(demo).\n-define(LOC, {?MODULE, ?FUNCTION_NAME, ?FUNCTION_ARITY}).\nf() -> ?LOC.\n",
        );
    }

    #[test]
    fn emit_atom_quotes_keywords_and_uppercase() {
        assert_eq!(emit_atom("foo"), "foo");
        assert_eq!(emit_atom("or"), "'or'");
        assert_eq!(emit_atom("Foo"), "'Foo'");
    }
}
