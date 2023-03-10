use crate::parser::{DefineVariable, Node, Operator2, Parameter, Type, REGISTER_SIZE};
use std::collections::HashMap;

pub struct Generator {
    next_label: usize,
}

#[derive(Debug)]
pub enum GenerateError {
    NotLeftValue,
    CallArgsOverFlow,
    DuplicatedVariable,
    UndefinedVariable(String),
    InvalidTypeSize(usize),
}

type GenerateResult = Result<(), GenerateError>;

const REGISTERS: [&'static str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

impl Generator {
    pub fn new() -> Self {
        Self { next_label: 0 }
    }

    fn assign_next_label(&mut self) -> usize {
        let r = self.next_label;
        self.next_label += 1;
        r
    }

    pub fn gen_lval(&mut self, node: &Node) -> GenerateResult {
        match node {
            Node::Deref(val) => {
                // println!("  ; deref gen_lval");
                self.gen(val)?;
                // println!("  pop rax");
                // println!("  mov rax [rax]");
                // println!("  push rax");
            }
            Node::LocalVariable(var) => {
                // println!("  ; local variable: {}", var.name());
                let offset = var.offset();

                println!("  mov rax, rbp");
                println!("  sub rax, {}", offset);
                println!("  push rax");
            }

            _ => {
                return Err(GenerateError::NotLeftValue);
            }
        }

        Ok(())
    }

    pub fn gen(&mut self, node: &Node) -> GenerateResult {
        dbg!(node);
        match node {
            Node::Num(n) => {
                println!("  push {}", n);
            }
            Node::LocalVariable(_a) => {
                self.gen_lval(node)?;
                println!("  pop rsi");
                println!("  mov rax, [rsi]");
                println!("  push rax");
            }
            Node::RelativeAddress(offset, _) => {
                println!("  mov rax, rbp");
                println!("  sub rax, {}", offset);
                println!("  push rax");
            }
            Node::DefineVariable(dv) => {
                println!("  sub rsp, {}", dv.ty().aligned_size());
                println!("  push rsp");
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
                        let left_type = left.declare_type().expect("type error");
                        dbg!(&left_type);

                        match left_type {
                            Type::Ptr(inner_type) => {
                                let sz = inner_type.size();
                                dbg!(sz);
                                match sz {
                                    4 => {
                                        println!("  shl rdi, 2");
                                    }
                                    8 => {
                                        println!("  shl rdi, 3");
                                    }
                                    _ => return Err(GenerateError::InvalidTypeSize(sz)),
                                }
                            }
                            _ => {}
                        }
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
            Node::Return(val) => {
                self.gen(val)?;
                println!("  pop rax");
                println!("  mov rsp, rbp");
                println!("  pop rbp");
                println!("  ret");
            }

            Node::IfElse(if_and_else) => {
                let if_label = self.assign_next_label();
                self.gen(if_and_else.condition().as_ref())?;
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .Lelse{}", if_label);
                self.gen(if_and_else.then_statement())?;
                println!("  jmp .Lend{}", if_label);
                println!(".Lelse{}:", if_label);
                if let Some(else_statement) = if_and_else.else_statement() {
                    self.gen(else_statement.as_ref())?;
                } else {
                    println!("  push 0");
                }
                println!(".Lend{}:", if_label);
            }
            Node::For(for_) => {
                let for_label = self.assign_next_label();
                if let Some(init) = for_.init() {
                    self.gen(init)?;
                }
                println!(".Lbegin{}:", for_label);
                if let Some(cond) = for_.cond() {
                    self.gen(cond)?;
                    println!("  pop rax");
                    println!("  cmp rax, 0");
                    println!("  je .Lend{}", for_label);
                }
                self.gen(for_.body())?;
                if let Some(next) = for_.next() {
                    self.gen(next)?;
                }
                println!("jmp .Lbegin{}", for_label);
                println!(".Lend{}:", for_label);
            }
            Node::Block(statements) => {
                for s in statements {
                    self.gen(s)?;
                    println!("  pop rax");
                }
            }

            Node::CallFunction(call_function) => {
                if call_function.args().len() > 6 {
                    return Err(GenerateError::CallArgsOverFlow);
                }
                for arg in call_function.args().iter().rev() {
                    self.gen(arg)?;
                }
                for (_, register) in call_function.args().iter().zip(REGISTERS) {
                    println!("  pop {}", register);
                }
                println!("  call {}", call_function.name());
                println!("  push rax");
            }
            Node::DefineFunction(define_function) => {
                println!("{}:", define_function.name());
                println!("  push rbp");
                println!("  mov rbp, rsp");
                for (register, param) in REGISTERS.iter().zip(define_function.params().iter()) {
                    println!("  push {}", register);
                }
                // println!("  sub rsp, {}", INTEGER_SIZE * 26); // FIXME(higumachan): ??????26?????????????????????????????????????????????????????????. ?????????????????????????????????????????????????????????,????????????????????????????????????.

                for statement in define_function.statements() {
                    self.gen(&statement)?;
                    println!("  pop rax");
                }

                println!("  mov rsp, rbp");
                println!("  pop rbp");
                println!("  ret");
            }
            Node::Addr(val) => {
                self.gen_lval(val.as_ref())?;
            }
            Node::Deref(val) => {
                // println!("  ; deref gen");
                self.gen(val.as_ref())?;
                println!("  pop rsi");
                println!("  mov rax, [rsi]");
                println!("  push rax");
            }
        }

        Ok(())
    }
}
