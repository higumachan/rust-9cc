extern crate core;

use std::env::args;
use std::str::FromStr;

fn main() {
    let argv: Vec<_> = args().collect();
    assert_eq!(args().len(), 2);

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let p = argv[1].as_str();
    let mut cs = p.chars().peekable();
    let num: String = {
        let mut s = String::new();

        while let Some(c) = cs.next_if(|c| c.is_ascii_digit()) {
            s.push(c);
        }
        s
    };
    println!("  mov rax, {}", num);

    while cs.peek().is_some() {
        let operator = cs.next().unwrap();
        let inst = match operator {
            '+' => "add",
            '-' => "sub",
            _ => panic!("invalid operator"),
        };
        let num: String = {
            let mut s = String::new();

            while let Some(c) = cs.next_if(|c| c.is_ascii_digit()) {
                s.push(c);
            }
            s
        };
        assert_ne!(num.len(), 0);

        println!("  {} rax, {}", inst, num);
    }
    println!("  ret");
}
