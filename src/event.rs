//! Internal event log used to build the flat syntax index.
//!
//! Grammar functions emit events as they run; each completed top-level unit
//! is finalized from these events into the syntax index.

use crate::syntax::SyntaxKind;
use crate::token_range::TokenIndex;

/// A single entry in the parser's event log.
///
/// The event log is grown by grammar functions and drained into the syntax
/// index at top-level unit boundaries. Both `Start` and `Finish` carry the
/// token-buffer position they were emitted at, which lets finalize compute
/// each entry's [`TokenRange`][crate::TokenRange] without replaying the
/// consumption history.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Event {
    /// Begin a syntax node.
    ///
    /// `kind` is `None` while the node is still tentative (its final kind
    /// is set at complete). `forward_parent`, when `Some(delta)`, indicates
    /// that another `Start` event at `self_index + delta` should be treated
    /// as this node's parent after finalize. This is how a completed node
    /// is retroactively wrapped by a new parent (Pratt-style promotion).
    /// `start_at` is the cursor position at emission time.
    Start {
        kind: Option<SyntaxKind>,
        forward_parent: Option<u32>,
        start_at: TokenIndex,
    },
    /// End the currently open node. `end_at` is the cursor position at
    /// emission time.
    Finish { end_at: TokenIndex },
    /// Placeholder left behind by an abandoned marker; skipped during
    /// finalize.
    Tombstone,
}
