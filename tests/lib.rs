extern crate erl_parse;
#[macro_use]
extern crate trackable;

use erl_parse::Parser;

#[test]
fn parse_hello_module() {
    let text = include_str!("hello.erl");
    let parser = track_try_unwrap!(Parser::new(text));
    let _module = track_try_unwrap!(parser.parse_module());
}
