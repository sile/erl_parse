//! Module-mode top-level driver.
//!
//! Called once per lexical `.` boundary by [`crate::Parser`]'s shared
//! dot-driven driver (`advance_dot_driven_grammar`). This module owns
//! only the grammar-side entry point: it parses one form via
//! [`crate::grammar::form::parse_form`]; the surrounding loop, dot
//! consumption, and unit-boundary finalization live in the parser
//! core so all three modes (Expression, Module, TermList) share the
//! same top-level machinery.
//!
//! The grand-root grouping (all forms wrapped in one node) is not
//! emitted — the parser emits one top-level unit per `.`-terminated
//! form and callers walk the syntax index unit by unit.

use crate::grammar::form::parse_form;
use crate::parser::{CompletedMarker, Parser};

/// Parses one top-level form in module mode. Consumes the form's
/// tokens up to (but not including) the terminating `.`.
pub(crate) fn parse_top_form(p: &mut Parser) -> CompletedMarker {
    parse_form(p)
}
