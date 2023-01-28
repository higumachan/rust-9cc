use crate::parser::{Node, Operator2};

pub struct Generator {}

#[derive(Debug)]
pub enum GenerateError {
    NotLeftValue,
}

type GenerateResult = Result<(), GenerateError>;

impl Generator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn gen_lval(&self, node: &Node) -> GenerateResult {
        let local_value = node.as_local_value().ok_or(GenerateError::NotLeftValue)?;

        println!("  mov rax, rbp");
        println!("  sub rax, {}", local_value.offset());
        println!("  push rax");

        Ok(())
    }

    pub fn gen(&self, node: &Node) -> GenerateResult {
        match node {
            Node::Num(n) => {
                println!("  push {}", n);
            }
            Node::LocalValue(a) => {
                self.gen_lval(node)?;
                println!("  pop rax");
                println!("  mov rax, [rax]");
                println!("  push rax");
            }
            Node::Assign { left, right } => {
                self.gen_lval(left.as_ref())?;
                self.gen(right.as_ref())?;

                println!("  pop rdi");
                println!("  pop rax");
                println!("  mov [rax], rdi");
                println!("  push rdi");
            }
            Node::Operator2 { op, left, right } => {
                self.gen(left.as_ref())?;
                self.gen(right.as_ref())?;

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

        Ok(())
    }
}
