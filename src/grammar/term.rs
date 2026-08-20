//! Term grammar.
//!
//! Recognises the token structure that Erlang treats as a term literal:
//! atoms, numbers, chars, strings, binary literals, tuples, proper /
//! improper lists, map literals, and unary sign prefixes. Variables,
//! calls, blocks, record literals, and bound identifiers are rejected,
//! aligning with what `file:consult/1` accepts via
//! `erl_parse:parse_term/1` → `erl_parse:normalise/1`.
//!
//! Rust-value conversion, Erlang term evaluation, and OTP Abstract
//! Format normalisation are out of scope.
