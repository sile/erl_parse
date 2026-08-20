//! Clause / body / guard grammar shared by block expressions and
//! function declarations.
//!
//! Block expressions (`case`, `if`, `receive`, `try`, `fun`, `maybe`)
//! and future function-declaration grammar (in the form / module family)
//! all group work into clauses whose shape is `Args [when Guard] ->
//! Body`. This module owns the shared implementation and exposes
//! `pub(crate)` helpers so the module and form grammar can reuse it
//! without duplicating clause / body / guard parsing.
