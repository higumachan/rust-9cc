use crate::parser::{Node, Operator2, INTEGER_SIZE};
use std::collections::HashMap;

struct LocalVariableAssigner {
    local_variables: HashMap<String, usize>,
    next_offset: usize,
}

impl LocalVariableAssigner {
    pub fn new() -> Self {
        Self {
            local_variables: HashMap::new(),
            next_offset: INTEGER_SIZE,
        }
    }

    fn clear(&mut self) {
        self.next_offset = INTEGER_SIZE;
        self.local_variables.clear();
    }

    pub fn assign_local_variable(&mut self, variable_name: &str) -> Option<usize> {
        if !self.local_variables.contains_key(variable_name) {
            let ret = self.next_offset;
            self.local_variables
                .insert(variable_name.to_string(), self.next_offset);
            self.next_offset += INTEGER_SIZE;
            Some(ret)
        } else {
            None
        }
    }

    pub fn get_or_assign_local_variable(&mut self, variable_name: &str) -> usize {
        *self
            .local_variables
            .entry(variable_name.to_string())
            .or_insert_with(|| {
                let ret = self.next_offset;
                self.next_offset += INTEGER_SIZE;
                ret
            })
    }
}

pub struct Generator {
    next_label: usize,
    local_variable_assigner: LocalVariableAssigner,
}

#[derive(Debug)]
pub enum GenerateError {
    NotLeftValue,
    CallArgsOverFlow,
    DuplicatedVariable,
}

type GenerateResult = Result<(), GenerateError>;

const REGISTERS: [&'static str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

impl Generator {
    pub fn new() -> Self {
        Self {
            next_label: 0,
            local_variable_assigner: LocalVariableAssigner::new(),
        }
    }

    fn assign_next_label(&mut self) -> usize {
        let r = self.next_label;
        self.next_label += 1;
        r
    }

    pub fn gen_lval(&mut self, node: &Node) -> GenerateResult {
        match node {
            Node::Deref(val) => {
                self.gen(val)?;
            }
            Node::LocalVariable(var) => {
                let offset = self
                    .local_variable_assigner
                    .get_or_assign_local_variable(var.name());

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
        match node {
            Node::Num(n) => {
                println!("  push {}", n);
            }
            Node::LocalVariable(_a) => {
                self.gen_lval(node)?;
                println!("  pop rax");
                println!("  mov rax, [rax]");
                println!("  push rax");
            }
            Node::DefineVariable(name) => {
                let _ = self
                    .local_variable_assigner
                    .assign_local_variable(name.as_str())
                    .ok_or(GenerateError::DuplicatedVariable);
                println!("  sub rsp, {}", INTEGER_SIZE);
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
                    println!("pop rax");
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
                self.local_variable_assigner.clear();
                println!("{}:", define_function.name());
                println!("  push rbp");
                println!("  mov rbp, rsp");
                for (register, param) in REGISTERS.iter().zip(define_function.params().iter()) {
                    println!("  push {}", register);
                    let _ = self
                        .local_variable_assigner
                        .assign_local_variable(param.as_str())
                        .ok_or(GenerateError::DuplicatedVariable)?;
                }
                println!("  sub rsp, {}", INTEGER_SIZE * 26); // FIXME(higumachan): 一旦26個のローカル変数用のスタックを用意する. 変数定義があるのでもうすでに必要ないが,互換性のために残している.

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
                self.gen(val.as_ref())?;
                println!("  pop rax");
                println!("  mov rax [rax]");
                println!("  push rax");
            }
        }

        Ok(())
    }
}
