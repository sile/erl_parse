erl_parse
=========

[![erl_parse](https://img.shields.io/crates/v/erl_parse.svg)](https://crates.io/crates/erl_parse)
[![Documentation](https://docs.rs/erl_parse/badge.svg)](https://docs.rs/erl_parse)
[![Actions Status](https://github.com/sile/erl_parse/workflows/CI/badge.svg)](https://github.com/sile/erl_parse/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

`erl_parse` is a Rust library for parsing Erlang source code, designed for
language tooling. It works directly on token streams:

- Callers provide the tokens; the crate does not read files, tokenize source, or
  preprocess it
- Every parse produces a syntax tree, with syntax problems reported as
  diagnostics
- The parser recovers from syntax errors so later forms, terms, or elements can
  still be parsed
- The grammar tracks OTP 29's `erl_parse.yrl`, and CI verifies it against
  OTP-29.0.5

Examples
--------

This example tokenizes a minimal Erlang module, feeds its tokens to the parser,
and collects the completed top-level nodes:

```rust
fn main() -> Result<(), erl_tokenize::Error> {
    let source = "-module(foo).";
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Module);
    for token in erl_tokenize::scan_tokens(source)? {
        parser.feed_token(token);
    }
    let mut roots = Vec::new();
    while let Some(id) = parser.next_node() {
        roots.push(id);
    }
    let tree = parser.finish();
    assert!(tree.diagnostics().is_empty());
    assert_eq!(roots.len(), 1);
    Ok(())
}
```

[erl_tokenize](https://github.com/sile/erl_tokenize) provides the tokens in this
example. Before parsing source that contains macros, includes, or conditionals,
preprocess it with [erl_pp](https://github.com/sile/erl_pp). See
[`otp_conformance`](examples/otp_conformance/) for a complete example of the
preprocessing and parsing pipeline.

[`ParseMode`](https://docs.rs/erl_parse/erl_parse/enum.ParseMode.html)
determines the kind of top-level construct the parser accepts. See
[Diagnostics and error recovery](docs/diagnostics.md) for recovery behavior and
[Walking a syntax tree](docs/navigation.md) for tree traversal.
