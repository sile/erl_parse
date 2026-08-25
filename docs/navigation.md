# Walking a syntax tree

A finished [`SyntaxTree`](crate::SyntaxTree) is the tokens the
caller passed plus a flat preorder array of nodes. Nothing in that
pair is a "current position". Forest-level questions live on the
tree; [`NodeView`](crate::NodeView) answers questions about **one
node**.

- [`SyntaxTree::roots`](crate::SyntaxTree::roots) lists the
  `.`-terminated units. [`SyntaxTree::nodes`](crate::SyntaxTree::nodes)
  walks every node, roots included.
  [`SyntaxTree::innermost_containing`](crate::SyntaxTree::innermost_containing)
  finds the tightest node around a token.
  [`SyntaxTree::view`](crate::SyntaxTree::view) wraps a
  [`NodeId`](crate::NodeId).
- [`NodeView`](crate::NodeView) is one node: its
  [`SyntaxKind`](crate::SyntaxKind) and
  [`TokenRange`](crate::TokenRange), its children, its descendants,
  its ancestors, and the tokens that sit in its range.

This is not a zipper. You do not park a cursor on a node and then
`goto_first_child` / `goto_next_sibling`. You ask the tree, then
you get `NodeView` values back. `NodeView` is `Copy`, so walking is
"hand me another view", not "mutate this one".

## The preorder array

The syntax index is one array in preorder. A parent occupies slot
`i`; its descendants occupy `i+1 .. subtree_end`. Sibling subtrees
are adjacent and do not overlap. There is no grand-root wrapping
the whole file: each `.`-terminated unit is its own root.

```text
source:  {1, 2}.

Tokens (every token the caller passed, hidden ones included):
  0 `{`   1 `1`   2 `,`   3 ` `   4 `2`   5 `}`   6 `.`

Preorder:
  0  TupleExpr          range covers `{` .. `}`
    1  IntegerExpr      token `1`
    2  IntegerExpr      token `2`
```

Punctuation and whitespace are **not** child nodes. They live on
the token buffer and show up through
[`NodeView::tokens`](crate::NodeView::tokens).
`children` / `descendants` only yield grammar nonterminals.

[`SyntaxTree::roots`](crate::SyntaxTree::roots) yields the
[`NodeView`](crate::NodeView) for slot `0` in that picture (the
unit root). Nested slots stay in the index; you walk them from the
root.

## After `parse`

Roots are `NodeView`s borrowed from the tree.

```rust
# fn main() -> Result<(), erl_tokenize::Error> {
let source = "{1, 2}.";
let tokens = erl_tokenize::scan_tokens(source)?;
let tree = erl_parse::parse(&tokens, erl_parse::ParseMode::Expression);

let roots: Vec<_> = tree.roots().collect();
assert_eq!(roots.len(), 1);
let root = roots[0];
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
    .iter()
    .position(|t| t.kind() == erl_tokenize::TokenKind::Integer)
    .map(erl_parse::TokenIndex::new);
let hit = one.and_then(|idx| tree.innermost_containing(idx));
assert_eq!(hit.map(|v| v.kind()), Some(erl_parse::SyntaxKind::IntegerExpr));
# Ok(())
# }
```

A [`NodeId`](crate::NodeId) or [`TokenIndex`](crate::TokenIndex) from
another tree is still the caller's problem: `roots` / `view` /
`innermost_containing` only keep this tree's buffer and index paired.

Several `.`-terminated units become several roots, in input order.

```rust
# fn main() -> Result<(), erl_tokenize::Error> {
let source = "{1}. {2}.";
let tokens = erl_tokenize::scan_tokens(source)?;
let tree = erl_parse::parse(&tokens, erl_parse::ParseMode::TermList);
let roots: Vec<_> = tree.roots().collect();
assert_eq!(roots.len(), 2);
assert_eq!(roots[0].kind(), erl_parse::SyntaxKind::TupleExpr);
assert_eq!(roots[1].kind(), erl_parse::SyntaxKind::TupleExpr);
# Ok(())
# }
```

[`SyntaxTree::view`](crate::SyntaxTree::view) returns `None` when
the id is past the end of the index. Ids from `roots` on that same
tree are always in range.

## Choosing a walk

| I want | Use |
| --- | --- |
| Each `.`-terminated unit | [`SyntaxTree::roots`](crate::SyntaxTree::roots) |
| Every node, preorder, roots included | [`SyntaxTree::nodes`](crate::SyntaxTree::nodes) |
| Direct children of one node | [`NodeView::children`](crate::NodeView::children) |
| Every nested node, preorder, excluding self | [`NodeView::descendants`](crate::NodeView::descendants) |
| Enclosing nodes, **innermost first** (direct parent toward the root) | [`NodeView::ancestors`](crate::NodeView::ancestors) |
| Tokens in this span, including whitespace and comments | [`NodeView::tokens`](crate::NodeView::tokens) |
| Tightest node whose non-empty range contains this token | [`SyntaxTree::innermost_containing`](crate::SyntaxTree::innermost_containing) |

A formatter or linter typically starts at `roots`, then
`children` / `descendants` filtered by `kind()`, or uses
[`SyntaxTree::nodes`](crate::SyntaxTree::nodes) when it wants every
node (roots included) in one pass. A hover or
click-to-node starts at `innermost_containing`. Reprinting a span
walks `tokens`, not `children`, so hidden tokens and
punctuation are not dropped.

[`NodeView::ancestors`](crate::NodeView::ancestors) starts at the
direct parent. The first item is the closest enclosing node; the
last item is the root that contains the node. The node itself is
not in the sequence.

## Reading a node's tokens

[`NodeView::tokens`](crate::NodeView::tokens) returns the raw tokens
inside a node's span as a slice, in buffer order. Hidden tokens
(whitespace and comments) are included, so the slice is 1:1 with the
node's [`TokenRange`](crate::TokenRange). It is the same buffer as
[`SyntaxTree::tokens`](crate::SyntaxTree::tokens):

```rust
# fn main() -> Result<(), erl_tokenize::Error> {
let source = "{1, 2}.";
let tokens = erl_tokenize::scan_tokens(source)?;
let tree = erl_parse::parse(&tokens, erl_parse::ParseMode::Expression);
let root = tree.roots().next().expect("one root");

assert_eq!(root.tokens(), &tree.tokens()[root.token_range().as_slice_index()]);
assert_eq!(root.tokens().len(), root.token_range().len());

// Zip the slice with its indices: a `TokenRange` iterates the
// `TokenIndex`es of the span.
let pairs: Vec<(erl_tokenize::Token, erl_parse::TokenIndex)> =
    root.tokens().iter().copied().zip(root.token_range()).collect();
assert_eq!(pairs.len(), root.token_range().len());
# Ok(())
# }
```

The first element sits at `token_range().start()`. When you keep a side
table parallel to [`SyntaxTree::tokens`](crate::SyntaxTree::tokens),
map the `i`-th element of the slice back to the buffer with
`TokenIndex::new(node.token_range().start().get() + i)`, or zip the slice
with the range itself: `node.tokens().iter().copied().zip(node.token_range())`
yields `(Token, TokenIndex)` pairs.

An `erl_tokenize::Token` alone has no spelling: it records where it
was scanned, not the source text. Pass the original source string to
`erl_tokenize::Token::text(source)` or
`erl_tokenize::Token::value(source)` to read the spelling or the
decoded value. The source must be the one the tokens were scanned
from; see the [erl_tokenize](https://docs.rs/erl_tokenize)
documentation for the contract.

## Empty ranges

A zero-width node is a real index entry: `children` can yield it,
and you can wrap its [`NodeId`](crate::NodeId). Its
[`NodeView::tokens`](crate::NodeView::tokens) slice is empty, while
`token_range().start()` still names the anchor position.
[`SyntaxTree::innermost_containing`](crate::SyntaxTree::innermost_containing)
never selects it, because an empty `[start, start)` does not
contain any [`TokenIndex`](crate::TokenIndex). Missing-token
recovery uses that shape; see
[`docs::diagnostics`](crate::docs::diagnostics).

## What they do not do

- They do not mutate the tree or the token buffer.
- Iterator-returning methods hand back opaque `impl Iterator`
  values. Name them with `for` / `.map` / `.collect`, not a
  concrete struct. The one concrete iterator is a
  [`TokenRange`](crate::TokenRange): it is `Copy` and iterates the
  span's [`TokenIndex`](crate::TokenIndex)es, like `std::ops::Range`.
- If you already have a [`NodeId`](crate::NodeId), start with
  [`SyntaxTree::view`](crate::SyntaxTree::view).
