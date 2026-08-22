erl_parse
=========

[![erl_parse](https://img.shields.io/crates/v/erl_parse.svg)](https://crates.io/crates/erl_parse)
[![Documentation](https://docs.rs/erl_parse/badge.svg)](https://docs.rs/erl_parse)
[![Actions Status](https://github.com/sile/erl_parse/workflows/CI/badge.svg)](https://github.com/sile/erl_parse/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Erlang source code parser written in Rust.

A parse always produces a syntax tree. Syntax problems are recorded as
diagnostics and the grammar recovers to the next sync point (typically
the next `.`) instead of aborting. See
[Diagnostics and error recovery](docs/diagnostics.md); the same page
renders as
[`docs::diagnostics`](https://docs.rs/erl_parse/erl_parse/docs/diagnostics/index.html)
on docs.rs.

Walking a finished tree uses `Cursor` and `NodeView`: the cursor
is the whole forest, a view is one node, and neither is a zipper. See
[Walking a syntax tree](docs/navigation.md); the same page renders as
[`docs::navigation`](https://docs.rs/erl_parse/erl_parse/docs/navigation/index.html)
on docs.rs.

References
----------

- [AST](http://erlang.org/doc/apps/erts/absform.html)
- [Macro](http://erlang.org/doc/reference_manual/macros.html)
- [Expression](http://erlang.org/doc/reference_manual/expressions.html)

Limitations
-----------

- Supports only UTF-8 source codes

Testing
-------

Unit tests and property tests do not need an Erlang runtime:

```text
cargo test --all
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
