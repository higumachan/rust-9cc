mod generator;
mod parser;

use crate::generator::Generator;
use crate::parser::TokenStream;
use crate::parser::{tokenize};
use std::env::args;








fn main() {
    let argv: Vec<_> = args().collect();
    assert_eq!(args().len(), 2);

    let p = argv[1].as_str();
    let tokens = tokenize(p).unwrap();

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
