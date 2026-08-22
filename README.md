erl_parse
=========

[![erl_parse](https://img.shields.io/crates/v/erl_parse.svg)](https://crates.io/crates/erl_parse)
[![Documentation](https://docs.rs/erl_parse/badge.svg)](https://docs.rs/erl_parse)
[![Actions Status](https://github.com/sile/erl_parse/workflows/CI/badge.svg)](https://github.com/sile/erl_parse/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Erlang source code parser written in Rust.

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

`check_otp_parse` treats preprocess-only failures as warnings. Parse errors
fail the process unless `OTP_PARSE_ERROR_MAX` is set higher than the error
file count. Unknown macros such as `?MODULE` are expanded to empty by the
driver, so a full OTP tree currently produces many parse errors; the smoke
job is still useful as a panic / completion check.
