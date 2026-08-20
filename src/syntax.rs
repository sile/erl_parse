//! Flat preorder syntax index.
//!
//! A [`SyntaxEntry`] carries a [`SyntaxKind`], a [`TokenRange`], and a
//! `subtree_end` ([`EntryIndex`]). A [`SyntaxIndex`] stores entries in
//! append-only preorder; interior insertion, deletion, reordering, and
//! in-place mutation are not exposed.
//!
//! See [`SyntaxIndex`]'s type documentation for the preorder-array
//! invariants that the builder must preserve.

use crate::token_range::TokenRange;

/// Grammar-level nonterminal kind assigned to a syntax entry.
///
/// This is a `Copy` enum that describes grammar nonterminals (module, form,
/// attribute, function declaration, clause, expression, pattern, type, and
/// so on). Terminal information (punctuation, keywords, literals) lives on
/// [`erl_tokenize::Token::kind`][erl_tokenize::Token::kind] and is not
/// duplicated as syntax entries.
///
/// The enum is not marked `#[non_exhaustive]`; adding a variant is treated
/// as a normal breaking change. Additional variants are introduced as
/// grammar coverage grows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    /// Marks a token range consumed by error recovery. The recovery logic
    /// itself is added by a subsequent change; this variant is present so
    /// consumers of the syntax index can navigate through the recovered
    /// range structurally.
    Error,

    // ---------------------------------------------------------------------
    // Expression / pattern / guard / term nodes.
    //
    // Structural node kinds intended for grammar output. Single-token
    // pieces (operators, punctuation, keywords) do not get their own
    // syntax entries; consumers read them directly through the token
    // buffer via the surrounding node's `TokenRange`. Pattern, guard, and
    // term positions reuse the expression kinds where the shape is
    // identical; parser-side allowlists enforce the position-specific
    // restrictions.
    //
    // Naming and structural inventory tracks OTP 29's
    // `lib/stdlib/src/erl_parse.yrl`; the productions this crate accepts
    // may lag or lead a specific OTP release as the language evolves.
    // ---------------------------------------------------------------------

    // Atomic expressions (single lexical token, occasionally with
    // adjacent-string concatenation).
    AtomExpr,
    VarExpr,
    IntegerExpr,
    FloatExpr,
    CharExpr,
    /// One or more adjacent string tokens concatenated at the syntactic
    /// level; the node's range spans every lexical string token in the
    /// run plus interior hidden tokens.
    StringExpr,
    SigilStringExpr,

    // Containers.
    TupleExpr,
    ListExpr,
    /// A list whose tail is not `[]`, written `[H1, H2, ... | Tail]`.
    ConsExpr,
    ParenExpr,
    BitstringExpr,
    MapExpr,
    /// A map update on an arbitrary expression, written `Expr#{...}`.
    MapUpdateExpr,
    RecordExpr,
    /// A record update, written `Expr#Name{...}`.
    RecordUpdateExpr,
    /// A record field access, written `Expr#Name.Field`.
    RecordFieldAccessExpr,
    /// A record field index, written `#Name.Field`.
    RecordIndexExpr,

    // Operations.
    /// A binary operator application (arithmetic, comparison, list,
    /// bitwise, boolean, etc.). The operator token is embedded in the
    /// node's range but is not a separate syntax entry.
    BinaryOpExpr,
    UnaryOpExpr,
    MatchExpr,
    SendExpr,
    /// A `maybe`-block match expression `X ?= Y`.
    MaybeMatchExpr,

    // Calls and remote references.
    CallExpr,
    /// A `Module:Function` reference used as a call target or a fun
    /// reference qualifier; interpreted by the parent node.
    RemoteExpr,

    // Blocks.
    BeginExpr,
    CatchExpr,
    CaseExpr,
    IfExpr,
    ReceiveExpr,
    TryExpr,
    MaybeExpr,

    // Fun expressions and references.
    /// `fun (Args) [when Guard] -> Body end`.
    AnonymousFun,
    /// `fun Name(Args) [when Guard] -> Body end`.
    NamedFun,
    /// `fun Name/Arity`.
    LocalFunRef,
    /// `fun Module:Name/Arity` (arms may be dynamic expressions).
    RemoteFunRef,

    // Comprehensions and qualifiers.
    ListComprehension,
    MapComprehension,
    BinaryComprehension,
    /// `Pat <- Expr`.
    Generator,
    /// `BinPat <= Expr`.
    BitstringGenerator,
    /// `Key := Value <- Expr`.
    MapGenerator,
    /// `Pat <:- Expr`.
    StrictGenerator,
    /// `BinPat <:= Expr`.
    StrictBitstringGenerator,
    /// `Key := Value <:- Expr`.
    StrictMapGenerator,
    /// Two or more generators joined by `&&`, forming a parallel
    /// (multi-valued) generator group.
    ZipQualifier,

    // Structural grouping used inside blocks, funs, and clauses.
    Body,
    Clause,
    IfClause,
    CatchClause,
    Guard,
    GuardSequence,
    ArgumentList,
    RecordField,
    MapField,
    BitstringElement,
}

/// Index into the entry array that identifies a boundary (values in
/// `0..=entries.len()`, so the trailing sentinel is representable).
///
/// Kept distinct from [`NodeId`], which refers to an existing entry:
/// using [`NodeId`] as a sentinel for "past the last entry" would blur
/// the distinction between element and boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntryIndex(usize);

impl EntryIndex {
    /// Constructs an `EntryIndex` from a raw index.
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the underlying `usize`.
    pub const fn get(self) -> usize {
        self.0
    }
}

/// Identifier of an existing syntax entry (values in `0..entries.len()`).
///
/// Kept distinct from [`EntryIndex`], which represents a boundary position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    /// Constructs a `NodeId` from a raw index.
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the underlying `usize`.
    pub const fn get(self) -> usize {
        self.0
    }

    /// Views the same position as an [`EntryIndex`].
    pub const fn as_entry_index(self) -> EntryIndex {
        EntryIndex(self.0)
    }
}

/// A single entry in the flat syntax index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxEntry {
    kind: SyntaxKind,
    range: TokenRange,
    subtree_end: EntryIndex,
}

impl SyntaxEntry {
    /// Constructs a `SyntaxEntry`.
    pub const fn new(kind: SyntaxKind, range: TokenRange, subtree_end: EntryIndex) -> Self {
        Self {
            kind,
            range,
            subtree_end,
        }
    }

    /// Returns the entry's [`SyntaxKind`].
    pub const fn kind(self) -> SyntaxKind {
        self.kind
    }

    /// Returns the token range covered by this entry.
    pub const fn range(self) -> TokenRange {
        self.range
    }

    /// Returns the boundary immediately past this entry's subtree in the
    /// preorder array.
    pub const fn subtree_end(self) -> EntryIndex {
        self.subtree_end
    }
}

/// Flat preorder array of syntax entries.
///
/// This type is not `Clone`; it is designed to be owned by a stateful
/// parser core. Entries can only be appended at the end; interior insertion,
/// deletion, reordering, and in-place mutation are not exposed.
///
/// # Preorder-array invariants
///
/// The builder is responsible for preserving the following invariants.
///
/// - A parent precedes every one of its descendants.
/// - For an entry at `self_index` ([`EntryIndex`]), `subtree_end` satisfies
///   `self_index + 1 <= subtree_end <= entries.len()`. For a leaf entry
///   (one with no descendants), `subtree_end == self_index + 1`. This is
///   independent of whether the node is zero-width.
/// - A child's subtree is fully contained within the parent's subtree.
/// - A child's [`TokenRange`] is contained within the parent's
///   [`TokenRange`].
/// - Sibling subtrees do not overlap.
/// - A parent and its single child may share the same [`TokenRange`]
///   (useful for constructs such as a completed marker in a Pratt parser
///   or an attribute wrapper that promotes an already-parsed subtree).
/// - Zero-width nodes (entries with an empty [`TokenRange`]) are allowed.
/// - Entries are appended at top-level unit boundaries. Interior entries
///   are not inserted mid-form, and entries are never reordered or removed
///   across top-level unit boundaries.
#[derive(Debug, Default, Clone)]
pub struct SyntaxIndex {
    entries: Vec<SyntaxEntry>,
}

impl SyntaxIndex {
    /// Creates an empty index.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Returns the number of entries currently held.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when the index has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the entry at `id`, or `None` when out of range.
    pub fn entry(&self, id: NodeId) -> Option<SyntaxEntry> {
        self.entries.get(id.get()).copied()
    }

    /// Borrows the internal slice.
    pub fn entries(&self) -> &[SyntaxEntry] {
        &self.entries
    }

    /// Returns the boundary [`EntryIndex`] past the last entry.
    pub fn end_index(&self) -> EntryIndex {
        EntryIndex::new(self.entries.len())
    }

    /// Converts an [`EntryIndex`] to a [`NodeId`] when it refers to an
    /// existing entry.
    pub fn node_id_at(&self, index: EntryIndex) -> Option<NodeId> {
        if index.get() < self.entries.len() {
            Some(NodeId::new(index.get()))
        } else {
            None
        }
    }

    /// Appends an entry to the end of the array and returns its [`NodeId`].
    ///
    /// The caller is responsible for preserving the invariants listed at
    /// the top of this type's documentation.
    pub(crate) fn push(&mut self, entry: SyntaxEntry) -> NodeId {
        let id = NodeId::new(self.entries.len());
        self.entries.push(entry);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_range::TokenIndex;

    fn range(start: usize, end: usize) -> TokenRange {
        TokenRange::new(TokenIndex::new(start), TokenIndex::new(end))
    }

    #[test]
    fn node_id_and_entry_index_are_distinct_types() {
        // NodeId (0..len) and EntryIndex (0..=len) are held in distinct
        // types. Conversion from NodeId to EntryIndex is direct; going the
        // other way requires SyntaxIndex to bounds-check.
        let node = NodeId::new(3);
        let boundary: EntryIndex = node.as_entry_index();
        assert_eq!(boundary.get(), 3);
    }

    #[test]
    fn append_only_builder() {
        let mut index = SyntaxIndex::new();
        assert!(index.is_empty());
        let a = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(0, 2),
            EntryIndex::new(1),
        ));
        let b = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(2, 4),
            EntryIndex::new(2),
        ));
        assert_eq!(a, NodeId::new(0));
        assert_eq!(b, NodeId::new(1));
        assert_eq!(index.len(), 2);
        assert_eq!(index.entry(a).map(|e| e.range()), Some(range(0, 2)));
        assert_eq!(index.entry(b).map(|e| e.range()), Some(range(2, 4)));
    }

    #[test]
    fn node_id_at_boundary_returns_none() {
        let mut index = SyntaxIndex::new();
        index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(0, 1),
            EntryIndex::new(1),
        ));
        assert_eq!(index.node_id_at(EntryIndex::new(0)), Some(NodeId::new(0)));
        assert_eq!(index.node_id_at(EntryIndex::new(1)), None);
        assert_eq!(index.end_index(), EntryIndex::new(1));
    }

    #[test]
    fn zero_width_entry_is_representable() {
        let mut index = SyntaxIndex::new();
        let id = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            TokenRange::empty_at(TokenIndex::new(3)),
            EntryIndex::new(1),
        ));
        let entry = index.entry(id).expect("id refers to an existing entry");
        assert!(entry.range().is_empty());
        assert_eq!(entry.range().start(), TokenIndex::new(3));
        assert_eq!(entry.range().end(), TokenIndex::new(3));
    }

    #[test]
    fn wrapper_and_child_share_range_but_subtree_size_differs() {
        // A parent (wrapper) and its single child may share the same
        // TokenRange. They are still distinguishable by their subtree size
        // (`subtree_end - self_index`): the parent covers itself plus the
        // child, the child alone.
        let mut index = SyntaxIndex::new();
        let same = range(0, 3);
        let parent_id = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            same,
            EntryIndex::new(2), // parent plus one child, so subtree_end is 2
        ));
        let child_id = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            same,
            EntryIndex::new(2), // leaf: self_index (1) + 1
        ));
        assert_ne!(parent_id, child_id);
        let parent = index
            .entry(parent_id)
            .expect("id refers to an existing entry");
        let child = index
            .entry(child_id)
            .expect("id refers to an existing entry");
        assert_eq!(parent.range(), child.range());
        // The parent occupies two entries (itself plus its one descendant).
        assert_eq!(parent.subtree_end().get() - parent_id.get(), 2);
        // The child is a leaf, so its subtree covers only itself.
        assert_eq!(child.subtree_end().get() - child_id.get(), 1);
    }
}
