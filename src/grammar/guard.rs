//! Guard grammar.
//!
//! Parses guard sequences (`Guard1 ; Guard2 ; ...`), each guard being a
//! comma-separated list of guard expressions. Structure only: whether a
//! given call is a guard BIF, whether a remote call is legal, and
//! whether operator operands are valid are semantic concerns handled by
//! an `erl_lint`-style pass, not by this parser.
