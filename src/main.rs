mod generator;
mod parser;

use crate::generator::Generator;
use crate::parser::TokenStream;
use crate::parser::{tokenize, Token};
use std::env::args;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::num::ParseIntError;
use std::slice::Iter;
use std::str::FromStr;
use std::vec::IntoIter;

fn main() {
    let argv: Vec<_> = args().collect();
    assert_eq!(args().len(), 2);

    let p = argv[1].as_str();
    let mut tokens = tokenize(p).unwrap();

    let mut token_stream = TokenStream::new(tokens);
    let node = token_stream.expr().unwrap();
    let generator = Generator::new();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    generator.gen(&node);

    println!("  pop rax");
    println!("  ret");
}
