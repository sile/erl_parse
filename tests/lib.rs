extern crate erl_parse;
extern crate erl_tokenize;
#[macro_use]
extern crate trackable;

use erl_tokenize::Tokenizer;
use erl_parse::Parser;

#[test]
fn parse_hello_module() {
    let text = include_str!("hello.erl");
    let tokenizer = Tokenizer::new(&text);
    let parser = Parser::new(tokenizer);
    let _module = track_try_unwrap!(parser.parse_module());
}
