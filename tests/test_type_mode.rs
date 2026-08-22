//! Integration tests for `erl_parse::ParseMode::Type`. Exercises the
//! type-expression grammar as external consumers see it.

fn scan_all(source: &str) -> Vec<erl_tokenize::Token> {
    let mut out = Vec::new();
    let mut pos = erl_tokenize::Position::new();
    while let Some(t) = erl_tokenize::scan_token(source, pos).expect("valid source") {
        out.push(t);
        pos = t.end();
    }
    out
}

fn feed_all(parser: &mut erl_parse::Parser, source: &str) {
    for t in scan_all(source) {
        parser.feed_token(t);
    }
}

fn kind_of(tree: &erl_parse::SyntaxTree, id: erl_parse::NodeId) -> erl_parse::SyntaxKind {
    tree.syntax().entry(id).expect("entry exists").kind()
}

fn drive(source: &str) -> (erl_parse::SyntaxTree, Vec<erl_parse::NodeId>) {
    let mut p = erl_parse::Parser::new(erl_parse::ParseMode::Type);
    feed_all(&mut p, source);
    let mut roots = Vec::new();
    while let Some(id) = p.next_node() {
        roots.push(id);
    }
    (p.finish(), roots)
}

#[test]
fn type_mode_emits_unit_on_dot() {
    let (tree, roots) = drive("foo.");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::AtomExpr);
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn type_mode_finish_flushes_input_without_trailing_dot() {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Type);
    feed_all(&mut parser, "list(integer())");
    assert!(parser.next_node().is_none());
    let tree = parser.finish();
    assert!(tree.diagnostics().is_empty());
    assert_eq!(
        kind_of(&tree, erl_parse::NodeId::new(0)),
        erl_parse::SyntaxKind::TypeCall
    );
}

#[test]
fn type_mode_emits_multiple_units_across_dots() {
    let (tree, roots) = drive("atom(). integer().");
    assert_eq!(roots.len(), 2);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::TypeCall);
    assert_eq!(kind_of(&tree, roots[1]), erl_parse::SyntaxKind::TypeCall);
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn type_mode_parses_union() {
    let (tree, roots) = drive("atom() | integer() | binary().");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::UnionType);
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn type_mode_parses_range() {
    let (tree, roots) = drive("1 .. 100.");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::RangeType);
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn type_mode_parses_annotated() {
    let (tree, roots) = drive("Var :: integer().");
    assert_eq!(roots.len(), 1);
    assert_eq!(
        kind_of(&tree, roots[0]),
        erl_parse::SyntaxKind::AnnotatedType
    );
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn type_mode_parses_function_type() {
    let (tree, roots) = drive("fun((atom(), integer()) -> boolean()).");
    assert_eq!(roots.len(), 1);
    assert_eq!(
        kind_of(&tree, roots[0]),
        erl_parse::SyntaxKind::FunctionType
    );
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn type_mode_parses_map_type() {
    let (tree, roots) = drive("#{atom() => integer()}.");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::MapType);
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn type_mode_parses_record_type() {
    let (tree, roots) = drive("#user{name :: binary()}.");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::RecordType);
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn type_mode_parses_bitstring_type() {
    let (tree, roots) = drive("<<_:8, _:_*4>>.");
    assert_eq!(roots.len(), 1);
    assert_eq!(
        kind_of(&tree, roots[0]),
        erl_parse::SyntaxKind::BitstringType
    );
    assert!(tree.diagnostics().is_empty());
}
