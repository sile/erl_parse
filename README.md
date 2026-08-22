erl_parse
=========

[![erl_parse](https://img.shields.io/crates/v/erl_parse.svg)](https://crates.io/crates/erl_parse)
[![Documentation](https://docs.rs/erl_parse/badge.svg)](https://docs.rs/erl_parse)
[![Actions Status](https://github.com/sile/erl_parse/workflows/CI/badge.svg)](https://github.com/sile/erl_parse/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Erlang source parser for language tools. The caller tokenizes
(`erl_tokenize::scan_token`) and feeds every token, including whitespace
and comments. This crate does not read files, tokenize, or preprocess.
A parse always produces a syntax tree; syntax problems are recorded as
diagnostics.

Examples
--------

```rust
fn main() {
    let source = "-module(foo).";
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Module);
    let mut pos = erl_tokenize::Position::new();
    while let Some(token) = erl_tokenize::scan_token(source, pos).expect("valid source") {
        parser.feed_token(token);
        pos = token.end();
    }
    let mut roots = Vec::new();
    while let Some(id) = parser.next_node() {
        roots.push(id);
    }
    let tree = parser.finish();
    assert!(tree.diagnostics().is_empty());
    assert_eq!(roots.len(), 1);
}
```

[`ParseMode`](https://docs.rs/erl_parse/erl_parse/enum.ParseMode.html)
selects the top-level construct. Recovery is
[Diagnostics and error recovery](docs/diagnostics.md)
([`docs::diagnostics`](https://docs.rs/erl_parse/erl_parse/docs/diagnostics/index.html)
on docs.rs). Walking the tree is
[Walking a syntax tree](docs/navigation.md)
([`docs::navigation`](https://docs.rs/erl_parse/erl_parse/docs/navigation/index.html)
on docs.rs).

Testing
-------

Unit tests and property tests do not need an Erlang runtime:

```text
cargo test --workspace
```

OTP grammar conformance lives in the `otp_conformance` workspace crate
(not in `cargo test`). Clone [erlang/otp](https://github.com/erlang/otp)
at the tag you want to compare, and pass that tag as `OTP_TAG` (CI sets
this in `.github/workflows/ci.yml`):

```text
export OTP_TAG=OTP-29.0.5
git clone --depth 1 --branch "$OTP_TAG" https://github.com/erlang/otp otp

# Smoke: tokenize, preprocess, and parse every target file
cargo run --release -p otp_conformance --bin check_otp_parse -- otp

# Differential: dump OTP erl_parse results, then compare
escript scripts/dump-otp-parse-fixture.escript otp > fixture.jsonl
cargo run --release -p otp_conformance --bin otp_diff -- fixture.jsonl
```

`check_otp_parse` treats preprocess-only failures as warnings. A file that
preprocesses cleanly but still has parse errors fails the process. The
driver expands OTP predefined macros (`?MODULE`, `?FUNCTION_NAME`,
`?OTP_RELEASE`, and so on); files that cannot be handled are listed in the
skip list rather than tolerated as parse errors.
