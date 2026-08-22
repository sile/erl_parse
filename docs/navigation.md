# Walking a syntax tree

A finished [`SyntaxTree`](crate::SyntaxTree) is a
[`TokenBuffer`](crate::TokenBuffer) plus a flat preorder
[`SyntaxIndex`](crate::SyntaxIndex). Nothing in that pair is a
"current position". The two public handles split the work:

- [`Cursor`](crate::Cursor) answers questions about the **whole
  forest**: which `.`-terminated units are roots, and which node is
  innermost around a given token.
- [`NodeView`](crate::NodeView) answers questions about **one
  node**: its [`SyntaxKind`](crate::SyntaxKind) and
  [`TokenRange`](crate::TokenRange), its children, its descendants,
  its ancestors, and the tokens that sit in its range.

Neither is a zipper. You do not park a cursor on a node and then
`goto_first_child` / `goto_next_sibling`. You build a `Cursor` from
the tree, then you get `NodeView` values back. `NodeView` is
`Copy`, so walking is "hand me another view", not "mutate this
one".

## The preorder array

The syntax index is one array in preorder. A parent occupies slot
`i`; its descendants occupy `i+1 .. subtree_end`. Sibling subtrees
are adjacent and do not overlap. There is no grand-root wrapping
the whole file: each `.`-terminated unit is its own root.

```text
source:  { 1 , ␠ 2 } .

TokenBuffer (every token the caller fed, hidden ones included):
  0 `{`   1 `1`   2 `,`   3 ` `   4 `2`   5 `}`   6 `.`

SyntaxIndex:
  0  TupleExpr          range covers `{` .. `}`
    1  IntegerExpr      token `1`
    2  IntegerExpr      token `2`
```

Punctuation and whitespace are **not** child nodes. They live on
the token buffer and show up through
[`NodeView::tokens_in_range`](crate::NodeView::tokens_in_range).
`children` / `descendants` only yield grammar nonterminals.

[`Parser::next_node`](crate::Parser::next_node) returns the
[`NodeId`](crate::NodeId) of slot `0` in that picture (the unit
root). Nested slots stay in the index; you walk them from the
root, you do not pull them one by one.

## After `finish`

Pair the tree's buffer with its index. That pair is a `Cursor`.
From there, roots are `NodeView`s.

```rust
let source = "{1, 2}.";
let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
let mut pos = erl_tokenize::Position::new();
while let Some(token) = erl_tokenize::scan_token(source, pos).expect("valid source") {
    parser.feed_token(token);
    pos = token.end();
}
let tree = parser.finish();

let cursor = erl_parse::Cursor::new(tree.tokens(), tree.syntax());
let root = cursor.roots().next().expect("one expression unit");
assert_eq!(root.kind(), erl_parse::SyntaxKind::TupleExpr);

let children: Vec<erl_parse::SyntaxKind> = root.children().map(|c| c.kind()).collect();
assert_eq!(
    children,
    [
        erl_parse::SyntaxKind::IntegerExpr,
        erl_parse::SyntaxKind::IntegerExpr
    ]
);

// Hit-test: the token that spells `1` sits inside the first integer node,
// not the tuple itself.
let one = tree
    .tokens()
    .as_slice()
    .iter()
    .position(|t| t.kind() == erl_tokenize::TokenKind::Integer)
    .map(erl_parse::TokenIndex::new)
    .expect("integer token");
let hit = cursor
    .innermost_containing(one)
    .expect("non-empty range contains the token");
assert_eq!(hit.kind(), erl_parse::SyntaxKind::IntegerExpr);
```

Use the same `tokens` and `syntax` that belong together. A
[`NodeId`](crate::NodeId) or [`TokenIndex`](crate::TokenIndex) from
one tree does not address another.

## During a pull parse

[`Parser::next_node`](crate::Parser::next_node) yields the same
root ids that [`Cursor::roots`](crate::Cursor::roots) will yield
after [`Parser::finish`](crate::Parser::finish). Wrap each id in a
[`NodeView`](crate::NodeView) while the parser is still alive, or
collect the ids and wrap them on the finished tree.

```rust
let source = "{1}. {2}.";
let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::TermList);
let mut pos = erl_tokenize::Position::new();
while let Some(token) = erl_tokenize::scan_token(source, pos).expect("valid source") {
    parser.feed_token(token);
    pos = token.end();
}

let mut root_ids = Vec::new();
while let Some(id) = parser.next_node() {
    root_ids.push(id);
}
let tree = parser.finish();
assert_eq!(root_ids.len(), 2);

let first = erl_parse::NodeView::new(tree.tokens(), tree.syntax(), root_ids[0])
    .expect("id came from next_node");
assert_eq!(first.kind(), erl_parse::SyntaxKind::TupleExpr);

let via_cursor: Vec<erl_parse::NodeId> = erl_parse::Cursor::new(tree.tokens(), tree.syntax())
    .roots()
    .map(|v| v.node_id())
    .collect();
assert_eq!(via_cursor, root_ids);
```

[`NodeView::new`](crate::NodeView::new) returns `None` when the id
is past the end of the index. Ids from `next_node` on that same
tree are always in range.

## Choosing a walk

| I want | Use |
| --- | --- |
| Each `.`-terminated unit | [`Cursor::roots`](crate::Cursor::roots) |
| Direct children of one node | [`NodeView::children`](crate::NodeView::children) |
| Every nested node, preorder, excluding self | [`NodeView::descendants`](crate::NodeView::descendants) |
| Enclosing nodes, **outermost first** (root toward the parent) | [`NodeView::ancestors`](crate::NodeView::ancestors) |
| Tokens in this span, including whitespace and comments | [`NodeView::tokens_in_range`](crate::NodeView::tokens_in_range) |
| Tightest node whose non-empty range contains this token | [`Cursor::innermost_containing`](crate::Cursor::innermost_containing) |

A formatter or linter typically starts at `roots`, then
`children` / `descendants` filtered by `kind()`. A hover or
click-to-node starts at `innermost_containing`. Reprinting a span
walks `tokens_in_range`, not `children`, so hidden tokens and
punctuation are not dropped.

[`NodeView::ancestors`](crate::NodeView::ancestors) does **not**
start at the parent. The first item is the root that contains the
node; the last item is the direct parent. The node itself is not
in the sequence.

## Empty ranges

A zero-width node is a real index entry: `children` can yield it,
and you can wrap its [`NodeId`](crate::NodeId). It yields no
tokens through `tokens_in_range`.
[`Cursor::innermost_containing`](crate::Cursor::innermost_containing)
never selects it, because an empty `[start, start)` does not
contain any [`TokenIndex`](crate::TokenIndex). Missing-token
recovery uses that shape; see
[`docs::diagnostics`](crate::docs::diagnostics).

## What they do not do

- They do not mutate the tree or the token buffer.
- Iterator-returning methods hand back opaque `impl Iterator`
  values. Name them with `for` / `.map` / `.collect`, not a
  concrete struct.
- There is no "current node" on [`Cursor`](crate::Cursor). If you
  already have a [`NodeId`](crate::NodeId), start with
  [`NodeView::new`](crate::NodeView::new).
- Kind and range alone do not need a view:
  [`SyntaxIndex::entry`](crate::SyntaxIndex::entry) is enough.
  Reach for [`NodeView`](crate::NodeView) when the walk also needs
  tokens or child/ancestor iterators.
