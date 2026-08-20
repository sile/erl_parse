//! Grammar functions built on top of the parser core.
//!
//! Each submodule implements one grammar family that reuses the parser
//! core's marker/cursor primitives:
//!
//! - [`operator`] holds the operator precedence table.
//! - [`expr`] parses expressions.
//! - [`pattern`] parses patterns (a restricted subset of expressions).
//! - [`guard`] parses guard sequences.
//! - [`term`] parses `file:consult/1`-style Erlang terms.
//! - [`ty`] parses Erlang type expressions.
//! - [`clause`] parses clause / body / guard groups shared by block
//!   expressions and function declarations.
//! - [`attribute`], [`function`], and [`form`] parse module-level
//!   forms; [`module`] and [`term_list`] wrap them as the top-level
//!   driver entry points for their respective parse modes.
//!
//! Grammar structure and precedence values track OTP 29's
//! `lib/stdlib/src/erl_parse.yrl`; the productions this crate accepts
//! may lag or lead a specific OTP release as the language evolves.

pub(crate) mod attribute;
pub(crate) mod clause;
pub(crate) mod expr;
pub(crate) mod form;
pub(crate) mod function;
pub(crate) mod guard;
pub(crate) mod module;
pub(crate) mod operator;
pub(crate) mod pattern;
pub(crate) mod recovery;
pub(crate) mod term;
pub(crate) mod term_list;
pub(crate) mod ty;
pub(crate) mod util;
