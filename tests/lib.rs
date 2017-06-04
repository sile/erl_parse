extern crate erl_parse;
#[macro_use]
extern crate trackable;

use erl_parse::{Parser, TokenRange};

macro_rules! parse_expr {
    ($text:expr) => {
        let parser = track_try_unwrap!(Parser::new($text));
        let expr = track_try_unwrap!(parser.parse_expr(), "text={:?}", $text);
        assert_eq!(expr.token_end(), parser.tokens().len());
    }
 }

#[test]
fn parse_expr_works() {
    // literals
    parse_expr!("foo");
    parse_expr!("$c");
    parse_expr!("1.2");
    parse_expr!("123");
    parse_expr!(r#""foo""#);

    // variable
    parse_expr!("Foo");

    // tuple
    parse_expr!("{}");
    parse_expr!("{1}");
    parse_expr!("{1, 2, 3}");

    // map
    parse_expr!("#{}");
    parse_expr!("#{a => b}");
    parse_expr!("#{a := b}");
    parse_expr!("#{a => b, 1 := 2}");

    // record
    parse_expr!("#foo{}");
    parse_expr!("#foo{a = b}");
    parse_expr!("#foo{a = b, _ = 10}");

    // record field index
    parse_expr!("#foo.bar");

    // proper list
    parse_expr!("[]");
    parse_expr!("[1]");
    parse_expr!("[1, 2, 3]");

    // improper list
    parse_expr!("[1 | 2]");
    parse_expr!("[1, 2 | 3]");

    // bitstring
    parse_expr!("<<>>");
    parse_expr!("<<10>>");
    parse_expr!("<<1, 2, 3>>");
    parse_expr!("<<100:2>>");
    parse_expr!("<<1/little>>");
    parse_expr!("<<1:2/little-unit:8>>");

    // block
    parse_expr!("begin 1, 2, 3 end");

    // parenthesized
    parse_expr!("( 1 )");

    // catch
    parse_expr!("catch [1,2,3]");

    // unary op
    parse_expr!("+10");
    parse_expr!("-20");
    parse_expr!("not false");
    parse_expr!("bnot Foo");

    // local fun
    parse_expr!("fun foo/2");

    // remote fun
    parse_expr!("fun foo:bar/2");
    parse_expr!("fun Foo:Bar/Baz");
}

macro_rules! parse_pattern {
    ($text:expr) => {
        let parser = track_try_unwrap!(Parser::new($text));
        let pattern = track_try_unwrap!(parser.parse_pattern(), "text={:?}", $text);
        assert_eq!(pattern.token_end(), parser.tokens().len());
    }
 }

#[test]
fn parse_pattern_works() {
    // literals
    parse_pattern!("foo");
    parse_pattern!("$c");
    parse_pattern!("1.2");
    parse_pattern!("123");
    parse_pattern!(r#""foo""#);

    // variable
    parse_pattern!("Foo");

    // bitstring
    parse_pattern!("<<>>");
    parse_pattern!("<<10>>");
    parse_pattern!("<<1, 2, 3>>");
    parse_pattern!("<<100:2>>");
    parse_pattern!("<<1/little>>");
    parse_pattern!("<<1:2/little-unit:8>>");

    // proper list
    parse_pattern!("[]");
    parse_pattern!("[1]");
    parse_pattern!("[1, 2, 3]");

    // improper list
    parse_pattern!("[1 | 2]");
    parse_pattern!("[1, 2 | 3]");

    // map
    parse_pattern!("#{}");
    parse_pattern!("#{a := b}");
    parse_pattern!("#{a := B, 1 := 2}");

    // tuple
    parse_pattern!("{}");
    parse_pattern!("{1}");
    parse_pattern!("{1, 2, 3}");

    // unary op
    parse_pattern!("+10");
    parse_pattern!("-20");

    // binary op
    parse_pattern!("[1] ++ [2,3]");

    // parenthesized
    parse_pattern!("( [1,2,3] )");

    // record
    parse_pattern!("#foo{}");
    parse_pattern!("#foo{a = b}");
    parse_pattern!("#foo{a = b, _ = 10}");

    // record field index
    parse_pattern!("#foo.bar");

    // match
    parse_pattern!("{A, B, 3} = {1, 2, 3}");
}

// #[test]
// fn parse_hello_module() {
//     let text = include_str!("hello.erl");
//     let parser = track_try_unwrap!(Parser::new(text));
//     let _module = track_try_unwrap!(parser.parse_module());
// }

// #[test]
// fn parse_fib_module() {
//     let text = include_str!("fib.erl");
//     let parser = track_try_unwrap!(Parser::new(text));
//     let _module = track_try_unwrap!(parser.parse_module());
// }

// #[test]
// fn parse_jsone_module() {
//     let text = include_str!("jsone.erl");
//     let parser = track_try_unwrap!(Parser::new(text));
//     let _module = track_try_unwrap!(parser.parse_module());
// }

// #[test]
// fn parse_splay_tree_module() {
//     let text = include_str!("splay_tree.erl");
//     let parser = track_try_unwrap!(Parser::new(text));
//     let _module = track_try_unwrap!(parser.parse_module());
// }
