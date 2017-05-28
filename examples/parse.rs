extern crate clap;
extern crate erl_parse;
extern crate erl_tokenize;
#[macro_use]
extern crate trackable;

use std::fs::File;
use std::io::Read;
use clap::{App, Arg};
use erl_parse::Parser;
use erl_tokenize::Tokenizer;
use trackable::error::{Failed, ErrorKindExt};

fn main() {
    let matches = App::new("parse")
        .arg(Arg::with_name("ERLANG_FILE").index(1).required(true))
        .get_matches();
    let erlang_file = matches.value_of("ERLANG_FILE").unwrap();

    let mut file = track_try_unwrap!(File::open(erlang_file).map_err(|e| Failed.cause(e)));
    let mut text = String::new();
    track_try_unwrap!(file.read_to_string(&mut text).map_err(|e| Failed.cause(e)));

    let tokenizer = Tokenizer::new(&text);
    let parser = Parser::new(tokenizer);
    let module = track_try_unwrap!(parser.parse_module());
    println!("{:?}", module);
}
