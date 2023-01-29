use crate::tokenizer::Token;
use std::collections::HashMap;

use std::fmt::{Display, Formatter};
use std::iter::Peekable;

use std::vec::IntoIter;

#[derive(Debug)]
pub enum ParseError {
    ExpectReserved(String),
    ExpectNumber,
    ExpectFunctionDefine,
}

impl Display for ParseError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub const INTEGER_SIZE: usize = 8;

pub type ParseResult<T> = std::result::Result<T, ParseError>;

pub struct TokenStream {
    inner: Peekable<IntoIter<Token>>,
    local_variables: HashMap<String, LocalVariable>,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            inner: tokens.into_iter().peekable(),
            local_variables: HashMap::new(),
        }
    }

    pub fn primary(&mut self) -> ParseResult<Node> {
        if self.consume_reserve("(") {
            let node = self.expr()?;
            self.expect(")")?;
            Ok(node)
        } else if let Some(ident_name) = self.consume_ident() {
            let ident_name = ident_name.clone();

            if self.consume_reserve("(") {
                let mut args = vec![];
                if !self.consume_reserve(")") {
                    args.push(self.expr()?);
                    while !self.consume_reserve(")") {
                        self.expect(",")?;
                        args.push(self.expr()?);
                    }
                }
                Ok(Node::CallFunction(CallFunction::new(ident_name, args)))
            } else {
                let lv = self
                    .local_variables
                    .entry(ident_name.clone())
                    .or_insert_with(move || {
                        let local_variable = LocalVariable::new(ident_name.to_string());

                        local_variable
                    })
                    .clone();
                Ok(Node::LocalVariable(lv))
            }
        } else {
            let number = self.expect_number()?;
            Ok(Node::Num(number))
        }
    }

    pub fn unary(&mut self) -> ParseResult<Node> {
        if self.consume_reserve("+") {
            self.primary()
        } else if self.consume_reserve("-") {
            Ok(Node::new_op2(
                Operator2::Sub,
                Box::new(Node::Num(0)),
                Box::new(self.primary()?),
            ))
        } else if self.consume_reserve("*") {
            Ok(Node::Deref(self.unary()?.into()))
        } else if self.consume_reserve("&") {
            Ok(Node::Addr(self.unary()?.into()))
        } else {
            self.primary()
        }
    }

    pub fn mul(&mut self) -> ParseResult<Node> {
        let mut node = self.unary()?;

        loop {
            node = if self.consume_reserve("*") {
                Node::new_op2(Operator2::Mul, Box::new(node), Box::new(self.unary()?))
            } else if self.consume_reserve("/") {
                Node::new_op2(Operator2::Div, Box::new(node), Box::new(self.unary()?))
            } else {
                break;
            }
        }

        Ok(node)
    }

    pub fn add(&mut self) -> ParseResult<Node> {
        let mut node = self.mul()?;

        loop {
            if self.consume_reserve("+") {
                let right = self.mul()?;
                node = Node::new_op2(Operator2::Add, Box::new(node), Box::new(right))
            } else if self.consume_reserve("-") {
                let right = self.mul()?;
                node = Node::new_op2(Operator2::Sub, Box::new(node), Box::new(right))
            } else {
                break;
            }
        }

        Ok(node)
    }

    pub fn relational(&mut self) -> ParseResult<Node> {
        let mut node = self.add()?;

        loop {
            if self.consume_reserve("<") {
                let right = self.add()?;
                node = Node::new_op2(Operator2::Lt, Box::new(node), Box::new(right))
            } else if self.consume_reserve("<=") {
                let right = self.add()?;
                node = Node::new_op2(Operator2::Lte, Box::new(node), Box::new(right))
            } else if self.consume_reserve(">") {
                let right = self.add()?;
                node = Node::new_op2(Operator2::Lt, Box::new(right), Box::new(node))
            } else if self.consume_reserve(">=") {
                let right = self.add()?;
                node = Node::new_op2(Operator2::Lte, Box::new(right), Box::new(node))
            } else {
                break;
            }
        }

        Ok(node)
    }

    pub fn equality(&mut self) -> ParseResult<Node> {
        let mut node = self.relational()?;

        loop {
            if self.consume_reserve("==") {
                let right = self.relational()?;
                node = Node::new_op2(Operator2::Eq, Box::new(node), Box::new(right))
            } else if self.consume_reserve("!=") {
                let right = self.relational()?;
                node = Node::new_op2(Operator2::Eq, Box::new(node), Box::new(right))
            } else {
                break;
            }
        }

        Ok(node)
    }

    pub fn assign(&mut self) -> ParseResult<Node> {
        let mut node = self.equality()?;
        if self.consume_reserve("=") {
            node = Node::new_assign(Box::new(node), Box::new(self.equality()?));
        }
        Ok(node)
    }

    pub fn expr(&mut self) -> ParseResult<Node> {
        self.assign()
    }

    pub fn statement(&mut self) -> ParseResult<Node> {
        if self.consume_reserve("{") {
            let mut statements = vec![];
            while !self.consume_reserve("}") {
                statements.push(self.statement()?);
            }
            Ok(Node::Block(statements))
        } else if self.consume_if() {
            self.expect("(")?;
            let cond = self.expr()?;
            self.expect(")")?;
            let then_statement = self.statement()?;
            let else_statement = if self.consume_else() {
                let else_statement = self.statement()?;
                Some(else_statement)
            } else {
                None
            };
            Ok(Node::IfElse(IfElse::new(
                cond.into(),
                then_statement.into(),
                else_statement.map(Box::new),
            )))
        } else if self.consume_for() {
            self.expect("(")?;
            let init = if !self.consume_reserve(";") {
                let init = self.expr()?;
                self.expect(";")?;
                Some(init)
            } else {
                None
            };
            let cond = if !self.consume_reserve(";") {
                let cond = self.expr()?;
                self.expect(";")?;
                Some(cond)
            } else {
                None
            };
            let next = if !self.consume_reserve(")") {
                let next = self.expr()?;
                self.expect(")")?;
                Some(next)
            } else {
                None
            };

            let body = self.statement()?;

            Ok(Node::For(For::new(
                init.map(Box::new),
                cond.map(Box::new),
                next.map(Box::new),
                body.into(),
            )))
        } else if self.consume_while() {
            self.expect("(")?;
            let cond = self.expr()?;
            self.expect(")")?;
            let body = self.statement()?;

            Ok(Node::For(For::new(
                None,
                Some(cond.into()),
                None,
                body.into(),
            )))
        } else {
            let is_return = self.consume_return();

            let node = self.expr()?;
            self.expect(";")?;

            if is_return {
                Ok(Node::Return(node.into()))
            } else {
                Ok(node)
            }
        }
    }

    fn param_list(&mut self) -> ParseResult<Vec<String>> {
        let mut params = vec![];
        self.expect("(")?;
        if !self.consume_reserve(")") {
            params.push(self.consume_ident().unwrap());
            while !self.consume_reserve(")") {
                self.expect(",")?;
                params.push(self.consume_ident().unwrap());
            }
        }
        Ok(params)
    }

    pub fn expect_function(&mut self) -> ParseResult<Node> {
        if let Some(name) = self.consume_ident() {
            let params = self.param_list()?;
            self.expect("{")?;
            let mut statements = vec![];
            while !self.consume_reserve("}") {
                statements.push(self.statement()?);
            }

            Ok(Node::DefineFunction(DefineFunction::new(
                name, params, statements,
            )))
        } else {
            Err(ParseError::ExpectFunctionDefine)
        }
    }

    pub fn program(&mut self) -> ParseResult<Vec<Node>> {
        let mut define_functions = vec![];

        while !self.at_eof() {
            define_functions.push(self.expect_function()?);
        }

        Ok(define_functions)
    }

    pub fn consume_ident(&mut self) -> Option<String> {
        match self.inner.peek().unwrap() {
            Token::Ident(n) => {
                let n = n.clone();
                self.inner.next().unwrap();
                Some(n)
            }
            _ => None,
        }
    }

    fn consume_if(&mut self) -> bool {
        match self.inner.peek().unwrap() {
            Token::If => {
                self.inner.next().unwrap();
                true
            }
            _ => false,
        }
    }

    fn consume_else(&mut self) -> bool {
        match self.inner.peek().unwrap() {
            Token::Else => {
                self.inner.next().unwrap();
                true
            }
            _ => false,
        }
    }

    fn consume_for(&mut self) -> bool {
        match self.inner.peek().unwrap() {
            Token::For => {
                self.inner.next().unwrap();
                true
            }
            _ => false,
        }
    }

    fn consume_while(&mut self) -> bool {
        match self.inner.peek().unwrap() {
            Token::While => {
                self.inner.next().unwrap();
                true
            }
            _ => false,
        }
    }

    fn consume_return(&mut self) -> bool {
        match self.inner.peek().unwrap() {
            Token::Return => {
                self.inner.next().unwrap();
                true
            }
            _ => false,
        }
    }

    pub fn consume_reserve(&mut self, op: &str) -> bool {
        match self.inner.peek().unwrap() {
            Token::Reserved(s) if s.as_str() == op => {
                self.inner.next().unwrap();
                true
            }
            _ => false,
        }
    }

    pub fn expect(&mut self, op: &str) -> ParseResult<()> {
        match self.inner.next().unwrap() {
            Token::Reserved(s) if s.as_str() == op => Ok(()),
            _ => Err(ParseError::ExpectReserved(op.to_string())),
        }
    }

    pub fn expect_number(&mut self) -> ParseResult<i64> {
        match self.inner.next().unwrap() {
            Token::Num(n) => Ok(n),
            _ => return Err(ParseError::ExpectNumber),
        }
    }

    pub fn at_eof(&mut self) -> bool {
        match self.inner.peek().unwrap() {
            Token::Eof => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum Operator2 {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Lte,
}

#[derive(Debug, Clone)]
pub struct LocalVariable {
    name: String,
}

impl LocalVariable {
    pub fn new(name: String) -> Self {
        Self { name }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct IfElse {
    condition: Box<Node>,
    then_statement: Box<Node>,
    else_statement: Option<Box<Node>>,
}

impl IfElse {
    pub fn new(
        condition: Box<Node>,
        then_statement: Box<Node>,
        else_statement: Option<Box<Node>>,
    ) -> Self {
        Self {
            condition,
            then_statement,
            else_statement,
        }
    }
    pub fn condition(&self) -> &Box<Node> {
        &self.condition
    }
    pub fn then_statement(&self) -> &Box<Node> {
        &self.then_statement
    }
    pub fn else_statement(&self) -> &Option<Box<Node>> {
        &self.else_statement
    }
}

#[derive(Debug)]
pub struct For {
    init: Option<Box<Node>>,
    cond: Option<Box<Node>>,
    next: Option<Box<Node>>,
    body: Box<Node>,
}

impl For {
    pub fn new(
        init: Option<Box<Node>>,
        cond: Option<Box<Node>>,
        next: Option<Box<Node>>,
        body: Box<Node>,
    ) -> Self {
        Self {
            init,
            cond,
            next,
            body,
        }
    }
    pub fn init(&self) -> &Option<Box<Node>> {
        &self.init
    }
    pub fn cond(&self) -> &Option<Box<Node>> {
        &self.cond
    }
    pub fn next(&self) -> &Option<Box<Node>> {
        &self.next
    }
    pub fn body(&self) -> &Box<Node> {
        &self.body
    }
}

#[derive(Debug)]
pub struct CallFunction {
    name: String,
    args: Vec<Node>,
}

impl CallFunction {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn args(&self) -> &Vec<Node> {
        &self.args
    }
    pub fn new(name: String, args: Vec<Node>) -> Self {
        Self { name, args }
    }
}

#[derive(Debug)]
pub struct DefineFunction {
    name: String,
    params: Vec<String>,
    statements: Vec<Node>,
}

impl DefineFunction {
    pub fn new(name: String, params: Vec<String>, statements: Vec<Node>) -> Self {
        Self {
            name,
            params,
            statements,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn params(&self) -> &Vec<String> {
        &self.params
    }
    pub fn statements(&self) -> &Vec<Node> {
        &self.statements
    }
}

#[derive(Debug)]
pub enum Node {
    Operator2 {
        op: Operator2,
        left: Box<Self>,
        right: Box<Self>,
    },
    Assign {
        left: Box<Self>,
        right: Box<Self>,
    },
    Addr(Box<Self>),
    Deref(Box<Self>),
    CallFunction(CallFunction),
    DefineFunction(DefineFunction),
    IfElse(IfElse),
    For(For),
    Return(Box<Self>),
    LocalVariable(LocalVariable),
    Num(i64),
    Block(Vec<Node>),
}

impl Node {
    pub fn new_op2(op: Operator2, left: Box<Self>, right: Box<Self>) -> Self {
        Self::Operator2 { op, left, right }
    }

    pub fn new_assign(left: Box<Self>, right: Box<Self>) -> Self {
        Self::Assign { left, right }
    }

    pub fn as_local_value(&self) -> Option<&LocalVariable> {
        match self {
            Self::LocalVariable(s) => Some(s),
            _ => None,
        }
    }
}
