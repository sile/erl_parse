//! Expression grammar.
//!
//! Reads `SyntaxKind` variants declared in [`crate::syntax`] using the
//! parser core's cursor and marker primitives. The core is reused by
//! [`crate::grammar::pattern`], [`crate::grammar::guard`], and
//! [`crate::grammar::term`] with position-specific allowlists layered on
//! top.
//!
//! Grammar shape follows OTP 29's `lib/stdlib/src/erl_parse.yrl`; see
//! [`crate::grammar::operator`] for the precedence table.
