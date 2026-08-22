//! Term-list mode top-level driver.
//!
//! Called once per lexical `.` boundary by [`crate::Parser`]'s shared
//! dot-driven driver. Parses one term via
//! [`crate::grammar::term::parse_term`] under
//! [`crate::parser::ParseContext::Term`]; the surrounding loop, dot
//! consumption, and unit-boundary finalization live in the parser
//! core so all four modes (Expression, Module, TermList, Type) share the
//! same top-level machinery.
//!
//! `parse_term` is invoked directly by this driver while a top-level
//! unit is already in progress.

use crate::grammar::term::parse_term;
use crate::parser::{CompletedMarker, Parser};

/// Parses one top-level term in term-list mode. Consumes the term's
/// tokens up to (but not including) the terminating `.`.
pub(crate) fn parse_top_term(p: &mut Parser) -> CompletedMarker {
    parse_term(p)
}
