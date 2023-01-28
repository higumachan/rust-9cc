mod generator;
mod parser;
mod tokenizer;

use crate::generator::Generator;
use crate::parser::TokenStream;
use crate::tokenizer::tokenize;
use std::env::args;

fn main() {
    let argv: Vec<_> = args().collect();
    assert_eq!(args().len(), 2);

    let p = argv[1].as_str();
    let tokens = tokenize(p).unwrap();

    let mut token_stream = TokenStream::new(tokens);
    let code = token_stream.program().unwrap();
    let mut generator = Generator::new();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, {}", 8 * 26);

    for line in &code {
        generator.gen(&line).unwrap();
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
