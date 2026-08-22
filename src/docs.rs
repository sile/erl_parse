//! Supplemental documentation modules that ship alongside the API
//! reference.
//!

// Document bodies live in `docs/` as Markdown and are pulled in with
// `include_str!` so this file stays a thin router.

/// Caller-facing contract for diagnostics and error recovery: a parse
/// always produces a tree, and the grammar resumes at the next sync
/// point rather than returning `Result::Err`.
#[doc = include_str!("../docs/diagnostics.md")]
pub mod diagnostics {}

/// How to walk a finished tree with [`Cursor`](crate::Cursor) and
/// [`NodeView`](crate::NodeView): `Cursor` is the forest,
/// `NodeView` is one node, and neither is a zipper.
#[doc = include_str!("../docs/navigation.md")]
pub mod navigation {}
