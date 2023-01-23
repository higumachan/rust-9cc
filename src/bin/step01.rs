use std::env::args;
use std::str::FromStr;

fn main() {
    let argv: Vec<_> = args().collect();
    assert_eq!(args().len(), 2);

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("  mov rax, {}", usize::from_str(argv[1].as_str()).unwrap());
    println!("  ret");
}
