erl_parse
=========

[![erl_parse](https://img.shields.io/crates/v/erl_parse.svg)](https://crates.io/crates/erl_parse)
[![Documentation](https://docs.rs/erl_parse/badge.svg)](https://docs.rs/erl_parse)
[![Actions Status](https://github.com/sile/erl_parse/workflows/CI/badge.svg)](https://github.com/sile/erl_parse/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Erlang source parser for language tools.

- You feed tokens; this crate does not read files, tokenize, or preprocess
- Whitespace and comments stay in the token stream
- A parse always produces a syntax tree; syntax problems are diagnostics
- Grammar tracks OTP 29 (`erl_parse.yrl`; CI checks against OTP-29.0.5)

Examples
--------

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

The snippet tokenizes with [erl_tokenize](https://github.com/sile/erl_tokenize).
For macros, includes, and conditionals, preprocess first with
[erl_pp](https://github.com/sile/erl_pp); [`otp_conformance`](examples/otp_conformance/) in this repo is an
end-to-end example of that pipeline.

[`ParseMode`](https://docs.rs/erl_parse/erl_parse/enum.ParseMode.html)
selects the top-level construct. Recovery is
[Diagnostics and error recovery](docs/diagnostics.md)
([`docs::diagnostics`](https://docs.rs/erl_parse/erl_parse/docs/diagnostics/index.html)
on docs.rs). Walking the tree is
[Walking a syntax tree](docs/navigation.md)
([`docs::navigation`](https://docs.rs/erl_parse/erl_parse/docs/navigation/index.html)
on docs.rs).
