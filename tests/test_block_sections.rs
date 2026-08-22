//! Integration tests for the section-node shape of `receive`, `try`,
//! and `maybe` block expressions. Every check goes through the public
//! `erl_parse::NodeView` / `erl_parse::SyntaxTree` navigation surface so it matches what
//! external consumers see.

fn scan_all(source: &str) -> Vec<erl_tokenize::Token> {
    let mut out = Vec::new();
    let mut pos = erl_tokenize::Position::new();
    while let Some(t) = erl_tokenize::scan_token(source, pos).expect("valid source") {
        out.push(t);
        pos = t.end();
    }
    out
}

/// Parses `source` in Expression mode (which uses the real block-
/// expression grammar) and returns the completed `erl_parse::SyntaxTree`.
fn parse_expr(source: &str) -> erl_parse::SyntaxTree {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
    for t in scan_all(source) {
        parser.feed_token(t);
    }
    parser.finish()
}

/// Returns a `erl_parse::NodeView` for the tree's first root node.
fn root_view(tree: &erl_parse::SyntaxTree) -> erl_parse::NodeView<'_> {
    tree.roots().next().expect("at least one root node")
}

fn child_kinds(view: erl_parse::NodeView<'_>) -> Vec<erl_parse::SyntaxKind> {
    view.children().map(|c| c.kind()).collect()
}

fn find_child(
    view: erl_parse::NodeView<'_>,
    kind: erl_parse::SyntaxKind,
) -> Option<erl_parse::NodeView<'_>> {
    view.children().find(|c| c.kind() == kind)
}

// -----------------------------------------------------------------
// receive
// -----------------------------------------------------------------

#[test]
fn receive_with_only_clauses_has_no_after_section() {
    let tree = parse_expr("receive msg -> ok end.");
    let receive = root_view(&tree);
    assert_eq!(receive.kind(), erl_parse::SyntaxKind::ReceiveExpr);
    assert!(!child_kinds(receive).contains(&erl_parse::SyntaxKind::ReceiveAfterSection));
}

#[test]
fn receive_with_only_after_wraps_the_after_section() {
    let tree = parse_expr("receive after 0 -> ok end.");
    let receive = root_view(&tree);
    assert_eq!(receive.kind(), erl_parse::SyntaxKind::ReceiveExpr);
    let after = find_child(receive, erl_parse::SyntaxKind::ReceiveAfterSection)
        .expect("after section is present");
    // Body plus the timeout expression live under the section.
    let inner: Vec<erl_parse::SyntaxKind> = after.children().map(|c| c.kind()).collect();
    assert!(
        inner.contains(&erl_parse::SyntaxKind::Body),
        "after section holds a Body: {inner:?}"
    );
    // The section's range must start on the `after` keyword itself, so
    // it is strictly narrower than the enclosing erl_parse::ReceiveExpr range.
    assert!(after.range().start() > receive.range().start());
    assert!(after.range().end() <= receive.range().end());
}

#[test]
fn receive_with_clauses_and_after_holds_both() {
    let tree = parse_expr("receive msg -> ok after 1000 -> timeout end.");
    let receive = root_view(&tree);
    assert_eq!(receive.kind(), erl_parse::SyntaxKind::ReceiveExpr);
    let kinds = child_kinds(receive);
    assert!(
        kinds.contains(&erl_parse::SyntaxKind::Clause),
        "clauses present: {kinds:?}"
    );
    assert!(
        kinds.contains(&erl_parse::SyntaxKind::ReceiveAfterSection),
        "after section present: {kinds:?}"
    );
}

// -----------------------------------------------------------------
// try
// -----------------------------------------------------------------

#[test]
fn try_with_of_catch_after_wraps_each_section() {
    let tree = parse_expr("try foo() of X -> X catch error:R -> R after cleanup() end.");
    let try_node = root_view(&tree);
    assert_eq!(try_node.kind(), erl_parse::SyntaxKind::TryExpr);
    let kinds = child_kinds(try_node);
    assert!(
        kinds.contains(&erl_parse::SyntaxKind::TryOfSection),
        "of section: {kinds:?}"
    );
    assert!(
        kinds.contains(&erl_parse::SyntaxKind::TryCatchSection),
        "catch section: {kinds:?}"
    );
    assert!(
        kinds.contains(&erl_parse::SyntaxKind::TryAfterSection),
        "after section: {kinds:?}"
    );
}

#[test]
fn try_of_clauses_and_pattern_only_catch_clauses_are_distinguishable() {
    // The `of` arm and the pattern-only `catch` arm both produce
    // `Clause` nodes; the section wrapper is what lets the caller
    // tell them apart.
    let tree = parse_expr("try foo() of X -> X catch Y -> Y end.");
    let try_node = root_view(&tree);
    let of_section = find_child(try_node, erl_parse::SyntaxKind::TryOfSection).expect("of section");
    let catch_section =
        find_child(try_node, erl_parse::SyntaxKind::TryCatchSection).expect("catch section");
    assert!(
        of_section
            .children()
            .any(|c| c.kind() == erl_parse::SyntaxKind::Clause)
    );
    assert!(
        catch_section
            .children()
            .any(|c| c.kind() == erl_parse::SyntaxKind::Clause)
    );
    assert!(of_section.range().end() <= catch_section.range().start());
}

#[test]
fn try_with_after_only_omits_of_and_catch_sections() {
    let tree = parse_expr("try do_thing() after cleanup() end.");
    let try_node = root_view(&tree);
    let kinds = child_kinds(try_node);
    assert!(!kinds.contains(&erl_parse::SyntaxKind::TryOfSection));
    assert!(!kinds.contains(&erl_parse::SyntaxKind::TryCatchSection));
    assert!(kinds.contains(&erl_parse::SyntaxKind::TryAfterSection));
}

#[test]
fn try_catch_section_holds_both_clause_and_catch_clause_shapes() {
    // Two clauses inside catch: a plain pattern clause and a
    // class-qualified catch clause. They coexist inside the same
    // erl_parse::TryCatchSection.
    let tree = parse_expr("try foo() catch Y -> Y; error:R:S -> R end.");
    let try_node = root_view(&tree);
    let catch = find_child(try_node, erl_parse::SyntaxKind::TryCatchSection).expect("catch");
    let inner: Vec<erl_parse::SyntaxKind> = catch.children().map(|c| c.kind()).collect();
    assert!(inner.contains(&erl_parse::SyntaxKind::Clause), "{inner:?}");
    assert!(
        inner.contains(&erl_parse::SyntaxKind::CatchClause),
        "{inner:?}"
    );
}

#[test]
fn try_catch_class_reason_can_be_a_match_pattern() {
    let tree = parse_expr("try foo() catch throw:{error, _} = E -> E end.");
    assert!(tree.diagnostics().is_empty(), "{:?}", tree.diagnostics());
    let try_node = root_view(&tree);
    let catch = find_child(try_node, erl_parse::SyntaxKind::TryCatchSection).expect("catch");
    let inner: Vec<erl_parse::SyntaxKind> = catch.children().map(|c| c.kind()).collect();
    assert!(
        inner.contains(&erl_parse::SyntaxKind::CatchClause),
        "{inner:?}"
    );
}

// -----------------------------------------------------------------
// maybe
// -----------------------------------------------------------------

#[test]
fn maybe_without_else_has_no_else_section() {
    let tree = parse_expr("maybe {ok, X} ?= foo() end.");
    let maybe = root_view(&tree);
    assert_eq!(maybe.kind(), erl_parse::SyntaxKind::MaybeExpr);
    assert!(!child_kinds(maybe).contains(&erl_parse::SyntaxKind::MaybeElseSection));
}

#[test]
fn maybe_with_else_wraps_the_else_section() {
    let tree = parse_expr("maybe {ok, X} ?= foo() else Other -> Other end.");
    let maybe = root_view(&tree);
    let else_section = find_child(maybe, erl_parse::SyntaxKind::MaybeElseSection)
        .expect("else section is present");
    // Body of the else clauses lives inside the section.
    assert!(
        else_section
            .children()
            .any(|c| c.kind() == erl_parse::SyntaxKind::Clause)
    );
    assert!(else_section.range().end() <= maybe.range().end());
}

// -----------------------------------------------------------------
// Range invariants shared by all sections.
// -----------------------------------------------------------------

#[test]
fn section_ranges_do_not_include_the_enclosing_end_keyword() {
    // For every optional section in a `try` block, the section's end
    // must lie at or before the parent's end, but never at exactly the
    // parent's `end` boundary; the parent still owns its own `end`
    // terminal.
    let tree = parse_expr("try foo() of X -> X catch Y -> Y after cleanup() end.");
    let try_node = root_view(&tree);
    let parent_end = try_node.range().end();
    for kind in [
        erl_parse::SyntaxKind::TryOfSection,
        erl_parse::SyntaxKind::TryCatchSection,
        erl_parse::SyntaxKind::TryAfterSection,
    ] {
        let section = find_child(try_node, kind).expect("section present");
        assert!(
            section.range().end() < parent_end,
            "{kind:?} range should stop before the enclosing `end`: {:?} vs parent end {:?}",
            section.range(),
            parent_end
        );
    }
}
