# Diagnostics and error recovery

erl_parse never aborts a parse with `Result::Err`. Pushing tokens and
calling [`Parser::finish`](crate::Parser::finish) always yields a
[`SyntaxTree`](crate::SyntaxTree). Syntax problems are recorded as
[`Diagnostic`](crate::Diagnostic)s on that tree, and the grammar
recovers to the next sync point so later forms, terms, or elements
can still land as ordinary nodes.

This page is the caller-facing contract: what recovery does, where it
resumes, and what it deliberately does not do. A compiler-like driver
and a formatter / linter can pick a policy from it without reading
the grammar.

## Two ways to consume a tree

**Strict.** Treat [`SyntaxTree::diagnostics`](crate::SyntaxTree::diagnostics)
being empty as success. The OTP smoke driver (`check_otp_parse`) uses
this: a file that preprocessed cleanly but still has diagnostics fails
the process.

**Best-effort.** Walk the syntax index and render the diagnostics
alongside it. Ranges consumed by recovery survive as
[`SyntaxKind::Error`](crate::SyntaxKind::Error) nodes and are
reachable through the same [`NodeView`](crate::NodeView) /
[`SyntaxTree`](crate::SyntaxTree) navigation surface as any other node (see
[`docs::navigation`](crate::docs::navigation)).
[`TokenRange`](crate::TokenRange) values on both diagnostics and
nodes are keys into [`SyntaxTree::tokens`](crate::SyntaxTree::tokens)
(and into any caller-side metadata table keyed the same way).

A `Diagnostic` is a record, not an operation-failure type: it does
not implement [`std::error::Error`], and the parser never returns it
as `Result::Err`. Callers read the records from the tree; they do not
construct them. Every diagnostic currently produced is a syntax
error; warnings and notes are not emitted yet.

There is no public "please recover" API. Recovery runs as
[`Parser::feed_token`](crate::Parser::feed_token),
[`Parser::next_node`](crate::Parser::next_node), and
[`Parser::finish`](crate::Parser::finish) drive the grammar.

## What happens at a problem site

Three recovery shapes cover the grammar:

**Missing token.** A required token is not there (a closing `)`, a
`end`, …). The parser records
[`DiagnosticKind::MissingToken`](crate::DiagnosticKind::MissingToken)
with a zero-width range at the cursor and does not advance. It does
not invent a fake [`erl_tokenize::Token`], so
[`SyntaxTree::tokens`](crate::SyntaxTree::tokens) stays a faithful
copy of what the caller fed. There is no `SyntaxKind::Error` node
for the missing token itself.

**Skip one token.** An atomic expression or type position sees a
token that cannot start that production. The parser consumes that one
lexical token, wraps it as `SyntaxKind::Error`, and records
[`DiagnosticKind::SkippedToken`](crate::DiagnosticKind::SkippedToken)
whose range matches the node's `TokenRange`.

**Skip until a sync token.** A production finished, but leftover
tokens sit before the token that would let the surrounding construct
resume. The parser consumes up to (not including) that sync token,
wraps the skipped span as `SyntaxKind::Error`, and records a matching
`SkippedToken`. The next iteration of the surrounding loop then sees
the sync token and continues.

A fourth case is not skip-recovery: **restricted positions.** Pattern
and `file:consult/1`-style term grammars reuse the expression tree
shape. A construct that is illegal there (a call in a pattern, a
variable in a term, …) still becomes a structured node; the parser
records [`DiagnosticKind::UnexpectedToken`](crate::DiagnosticKind::UnexpectedToken)
and does not stall the cursor.

## Where recovery resumes

Sync points are local to the construct that is open:

| Open construct | Resume at |
| --- | --- |
| Module form, top-level expression, term, or type | the next `.` that terminates that unit |
| Tuple, list, map, record, bitstring, or argument list | the next `,` or the closing delimiter |
| `case` / `receive` / `try` / `fun` / `maybe` / function-decl clauses | the next `;`, or `end` / `after` / `catch` / `else` |

Consequences callers actually see:

- A malformed module form does not swallow the rest of the file.
  `1 2 3. -ok.` still yields an `Attribute` node for `-ok.`.
- A malformed list element does not swallow the rest of the list.
  `[1, ), 3].` still has an integer `3` in the syntax index.
- A garbled clause head does not swallow the rest of the block. The
  next `;` or `end` is where the next clause starts.
- Several independent diagnostics can come out of one module; they
  are separate records, not one aggregated blob.

The same recovery site will not re-run at the same cursor position
without the cursor moving. A stuck production therefore cannot loop
on skip-recovery; it either advances or stops skipping.

Adjacent appends of the same [`DiagnosticKind`](crate::DiagnosticKind)
at the same [`TokenRange::start`](crate::TokenRange::start) are
collapsed to one entry, so a recovery loop that tries several
alternatives at one position does not surface the same diagnostic
twice.

## `SyntaxKind::Error` versus `Diagnostic`

They are related but not 1:1.

- A `SkippedToken` diagnostic matches an `Error` node with the same
  `TokenRange`. Walk either direction: diagnostic → node, or node →
  diagnostic.
- `MissingToken` is a boundary marker only: zero-width range, no
  `Error` node.
- [`UnexpectedToken`](crate::DiagnosticKind::UnexpectedToken) /
  [`UnexpectedEof`](crate::DiagnosticKind::UnexpectedEof) may sit on
  a zero-width `Error` node (nothing to parse at this production) or
  on a force-closed unit (see [End of input](#end-of-input) below).
- [`NestingDepthExceeded`](crate::DiagnosticKind::NestingDepthExceeded)
  is a zero-width boundary diagnostic. The parser stops descending
  past [`Parser::MAX_NESTING_DEPTH`](crate::Parser::MAX_NESTING_DEPTH)
  (256) and continues recovery from there instead of overflowing the
  stack.

Tokenizer / lexer failures never appear as parser diagnostics. The
caller tokenizes ([`erl_tokenize::scan_token`]) and feeds tokens;
only `DiagnosticKind` variants this crate owns land on the tree.

## End of input

Top-level units are `.`-terminated.
[`Parser::finish`](crate::Parser::finish) still parses leftover
lexical tokens when the buffer ended without a `.`, as one last unit
for the mode. Diagnostics for a missing `.` or a missing closer flow
into the tree as usual.

If a top-level unit is still open at `finish`, it is force-closed as
`SyntaxKind::Error` covering the unterminated span, with a matching
`UnexpectedEof` diagnostic on the same range.

## Example

A first form that is garbage, then a well-formed attribute. The
second form still lands; the tree carries diagnostics for the first.

```rust
use erl_tokenize::{Position, scan_token};

let source = "1 2 3.\n-ok.";
let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Module);
let mut pos = Position::new();
while let Some(token) = scan_token(source, pos).expect("valid source") {
    parser.feed_token(token);
    pos = token.end();
}

let mut roots = Vec::new();
while let Some(id) = parser.next_node() {
    roots.push(id);
}
let tree = parser.finish();

assert!(roots.len() >= 2);
assert!(!tree.diagnostics().is_empty());
let last = *roots.last().expect("last root");
assert_eq!(
    tree.syntax().entry(last).expect("entry").kind(),
    erl_parse::SyntaxKind::Attribute,
);
```

A strict caller would treat that tree as a failed parse
(`!diagnostics().is_empty()`). A best-effort caller would still walk
`roots[1]` as `-ok.`.

## What this crate does not do

- Invent tokens to "repair" the input.
- Expose a skip-to-next-form or rewind API. Continue driving
  `feed_token` / `next_node`, or drop the [`Parser`](crate::Parser).
- Emit warnings or notes. Every diagnostic is currently an error.
- Report preprocessor or tokenizer problems. Those belong to `erl_pp`
  / `erl_tokenize` (and to the driver that sits between them).
- Guarantee that a recovered tree is a valid Erlang program. Recovery
  keeps later units and later elements reachable; it does not make a
  broken construct well-formed.

## Guarantees

Within the design above:

- [`Parser::finish`](crate::Parser::finish) returns a `SyntaxTree` for
  every input the caller fed.
- Hidden tokens around recovered spans stay in
  [`SyntaxTree::tokens`](crate::SyntaxTree::tokens).
- A `SkippedToken` diagnostic's range equals the corresponding
  `Error` node's range.
- A `MissingToken` diagnostic's range is empty, and the token count
  of the tree equals the number of tokens the caller fed.
- Hitting the nesting-depth cap surfaces `NestingDepthExceeded`
  rather than panicking or overflowing the stack.
- Recovery at a given site makes forward progress or refuses to
  re-enter, so parse of a finite token buffer terminates.

Anything outside these bullets is either a documented restriction or
a bug.

**See also.** [`Diagnostic`](crate::Diagnostic),
[`DiagnosticKind`](crate::DiagnosticKind),
[`SyntaxKind::Error`](crate::SyntaxKind::Error),
[`SyntaxTree::diagnostics`](crate::SyntaxTree::diagnostics),
[`Parser::finish`](crate::Parser::finish),
[`Parser::MAX_NESTING_DEPTH`](crate::Parser::MAX_NESTING_DEPTH).
