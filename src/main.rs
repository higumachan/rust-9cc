use std::env::args;
use std::str::FromStr;

fn main() {
    let argv: Vec<_> = args().collect();
    assert!(args().len() == 2);

    println!(".intel_sntax noprefix");
    println!(".globl main\n");
    println!("main:\n");
    println!("  mov rax, {}\n", usize::from_str(argv[1].as_str()).unwrap());
    println!("  ret\n");
}
