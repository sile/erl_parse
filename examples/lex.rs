extern crate clap;
extern crate erl_parse;

use std::fs;
use std::io::Read;
use clap::{App, Arg};

fn main() {
    let matches =
        App::new("lex").arg(Arg::with_name("SOURCE_FILE").index(1).required(true)).get_matches();
    let source_file = matches.value_of("SOURCE_FILE").unwrap();
    let mut source = String::new();
    fs::File::open(source_file)
        .expect("Can not open file")
        .read_to_string(&mut source)
        .expect("Can not read source file");

    let lexer = erl_parse::lexer::Lexer::new(&source);
    for token in lexer.tokenize().expect("Failed to tokenize") {
        println!("{:?}", token);
    }
}
