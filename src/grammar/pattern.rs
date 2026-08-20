//! Pattern grammar.
//!
//! Shares the token cursor, precedence table, and marker primitives with
//! [`crate::grammar::expr`], but restricts the accepted productions to
//! the subset that is valid on the left-hand side of a match: literals,
//! variables, container patterns, bitstring patterns, record / map
//! patterns, `MatchPattern`, and the literal arithmetic allowed by OTP
//! 29's `pat_expr` production. Call, block, and general expression
//! constructs are rejected.
