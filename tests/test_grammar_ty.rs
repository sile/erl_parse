//! Integration tests for the type-grammar auxiliary entry point
//! `erl_parse::Parser::parse_type_range`. Exercises the public surface as
//! external consumers see it.

fn scan_all(source: &str) -> Vec<erl_tokenize::Token> {
    let mut out = Vec::new();
    let mut pos = erl_tokenize::Position::new();
    while let Some(t) = erl_tokenize::scan_token(source, pos).expect("valid source") {
        out.push(t);
        pos = t.end();
    }
    out
}

/// Pushes `source` (without a trailing `.`) into a fresh Expression-
/// mode parser, then invokes `parse_type_range` on the whole buffer
/// and returns a snapshot of the resulting `erl_parse::SyntaxTree` plus the
/// `erl_parse::NodeId` of the produced type node.
///
/// Expression mode only opens a top-level unit when it sees a `.`,
/// so with a source that has none the aux entry point's precondition
/// (no in-progress unit) is satisfied at push time. We snapshot the
/// tree via `Clone` rather than calling `finish` because
/// `parse_type_range` restores the cursor after appending the unit,
/// which would let `finish` (in Expression mode) re-parse the same
/// tokens as an expression and add spurious siblings.
fn parse_type(source: &str) -> (erl_parse::SyntaxTree, erl_parse::NodeId) {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
    for t in scan_all(source) {
        parser.push_token(t);
    }
    let range = erl_parse::TokenRange::new(
        erl_parse::TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let id = parser
        .parse_type_range(range)
        .expect("no in-progress unit for `.`-less input");
    (parser.syntax_tree().clone(), id)
}

fn kind_of(tree: &erl_parse::SyntaxTree, id: erl_parse::NodeId) -> erl_parse::SyntaxKind {
    tree.syntax().entry(id).expect("entry exists").kind()
}

#[test]
fn parse_type_range_returns_atom_type_for_bare_atom() {
    let (tree, id) = parse_type("foo");
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::AtomExpr);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_type_range_returns_type_call_for_name_with_arguments() {
    let (tree, id) = parse_type("list(integer())");
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::TypeCall);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_type_range_returns_union_for_bar_separated_types() {
    let (tree, id) = parse_type("atom() | integer() | binary()");
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::UnionType);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_type_range_returns_range_for_double_dot() {
    let (tree, id) = parse_type("1 .. 100");
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::RangeType);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_type_range_returns_annotated_for_double_colon() {
    let (tree, id) = parse_type("Var :: integer()");
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::AnnotatedType);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_type_range_returns_function_type_for_fun_arrow() {
    let (tree, id) = parse_type("fun((atom(), integer()) -> boolean())");
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::FunctionType);
    assert!(
        tree.errors().is_empty(),
        "unexpected errors: {:?}",
        tree.errors()
    );
}

#[test]
fn parse_type_range_returns_map_type_for_hash_brace() {
    let (tree, id) = parse_type("#{atom() => integer()}");
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::MapType);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_type_range_returns_record_type_for_hash_name_brace() {
    let (tree, id) = parse_type("#user{name :: binary()}");
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::RecordType);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_type_range_returns_bitstring_type() {
    let (tree, id) = parse_type("<<_:8, _:_*4>>");
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::BitstringType);
    assert!(tree.errors().is_empty());
}
