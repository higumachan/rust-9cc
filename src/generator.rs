use crate::parser::{Node, Operator2};

pub fn gen(_node: &Node) {}

pub struct Generator {}

impl Generator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn gen(&self, node: &Node) {
        match node {
            Node::Num(n) => {
                println!("  push {}", n);
            }
            Node::Operator2 { op, left, right } => {
                gen(left.as_ref());
                gen(right.as_ref());
                println!("  pop rdi");
                println!("  pop rax");

                match op {
                    Operator2::Add => {
                        println!("  add rax, rdi");
                    }
                    Operator2::Sub => {
                        println!("  sub rax, rdi");
                    }
                    Operator2::Mul => {
                        println!("  mul rdi");
                    }
                    Operator2::Div => {
                        println!("  cqo");
                        println!("  idiv rdi");
                    }
                    Operator2::Eq => {
                        println!("  cmp rax, rdi");
                        println!("  sete al");
                        println!("  movzb rax, al");
                    }
                    Operator2::Ne => {
                        println!("  cmp rax, rdi");
                        println!("  setne al");
                        println!("  movzb rax, al");
                    }
                    Operator2::Lt => {
                        println!("  cmp rax, rdi");
                        println!("  setl al");
                        println!("  movzb rax, al");
                    }
                    Operator2::Lte => {
                        println!("  cmp rax, rdi");
                        println!("  setle al");
                        println!("  movzb rax, al");
                    }
                }

                println!("  push rax");
            }
        }
    }
}
