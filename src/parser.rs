use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::FromStr;
use std::vec::IntoIter;

#[derive(Debug)]
pub enum Token {
    Reserved(String),
    Ident(String),
    Num(i64),
    Return,
    Eof,
}

impl Token {
    pub fn reserved(s: &str) -> Self {
        Token::Reserved(s.to_string())
    }

    pub fn as_reserved(&self) -> Option<&String> {
        match self {
            Token::Reserved(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_num(&self) -> Option<i64> {
        match self {
            Token::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_eof(&self) -> bool {
        match self {
            Token::Eof => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    ExpectReserved(String),
    ExpectNumber,
}

impl Display for ParseError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

const INTEGER_SIZE: usize = 8;

pub type ParseResult<T> = std::result::Result<T, ParseError>;

pub struct TokenStream {
    inner: Peekable<IntoIter<Token>>,
    local_variables: HashMap<String, LocalVariable>,
    next_offset: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            inner: tokens.into_iter().peekable(),
            local_variables: HashMap::new(),
            next_offset: INTEGER_SIZE,
        }
    }

    pub fn primary(&mut self) -> ParseResult<Node> {
        if self.consume_reserve("(") {
            let node = self.expr()?;
            self.expect(")")?;
            Ok(node)
        } else if let Some(ident_name) = self.consume_ident() {
            let ident_name = ident_name.clone();
            let next_offset = &mut self.next_offset;
            let lv = self
                .local_variables
                .entry(ident_name.clone())
                .or_insert_with(move || {
                    let local_variable = LocalVariable::new(ident_name.to_string(), *next_offset);
                    *next_offset += INTEGER_SIZE;

                    local_variable
                })
                .clone();
            Ok(Node::LocalVariable(lv))
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
        let is_return = self.consume_return();

        let node = self.expr()?;
        self.expect(";")?;

        if is_return {
            Ok(Node::Return(node.into()))
        } else {
            Ok(node)
        }
    }

    pub fn program(&mut self) -> ParseResult<Vec<Node>> {
        let mut lines = vec![];
        while !self.at_eof() {
            lines.push(self.statement()?);
        }

        Ok(lines)
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

pub fn parse_number(
    iter: &mut Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<i64, GeneralError> {
    let mut s = String::new();

    while let Some((_pos, c)) = iter.next_if(|(_pos, c)| c.is_ascii_digit()) {
        s.push(c);
    }
    i64::from_str(s.as_str()).map_err(|_| GeneralError::new("整数がパースできません".to_string()))
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
    offset: usize,
}

impl LocalVariable {
    pub fn new(name: String, offset: usize) -> Self {
        Self { name, offset }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn offset(&self) -> usize {
        self.offset
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
    Return(Box<Self>),
    LocalVariable(LocalVariable),
    Num(i64),
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

#[derive(Debug)]
pub struct GeneralError {
    message: String,
}

impl GeneralError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Display for GeneralError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for GeneralError {}

#[derive(Debug)]
pub struct TokenizeError {
    message: String,
    line_number: usize,
    source_code_line: String,
    pos: usize,
}

impl TokenizeError {
    pub fn new(message: String, line_number: usize, source_code_line: String, pos: usize) -> Self {
        Self {
            message,
            line_number,
            source_code_line,
            pos,
        }
    }
}

impl Display for TokenizeError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for TokenizeError {}

pub type TokenizeResult<T> = std::result::Result<T, TokenizeError>;

fn match_string<T: Iterator<Item = (usize, char)> + Clone>(p_iter: &T, s: &str) -> bool {
    let p_iter = p_iter.clone();
    s.chars()
        .zip(p_iter.take(s.len()))
        .all(|(c1, (_, c2))| c1 == c2)
}

fn match_variable_string<T: Iterator<Item = (usize, char)> + Clone>(p_iter: &T) -> Option<String> {
    let mut p_iter2 = p_iter.clone();

    if let Some((_, c)) = p_iter2.next() {
        if c.is_ascii_alphabetic() {
            let mut s = c.to_string();
            s.extend(
                p_iter2
                    .map(|(_, c)| c)
                    .take_while(|c| c.is_ascii_alphanumeric()),
            );
            Some(s)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn tokenize(input: &str) -> TokenizeResult<Vec<Token>> {
    let mut cs = input.chars().enumerate().peekable();
    let mut tokens = vec![];

    while let Some((pos, c)) = cs.peek().cloned() {
        if match_string(&cs, "==") {
            tokens.push(Token::Reserved("==".to_string()));
            cs.next();
            cs.next();
        } else if match_string(&cs, "!=") {
            tokens.push(Token::Reserved("!=".to_string()));
            cs.next();
            cs.next();
        } else if match_string(&cs, ">=") {
            tokens.push(Token::Reserved(">=".to_string()));
            cs.next();
            cs.next();
        } else if match_string(&cs, "<=") {
            tokens.push(Token::Reserved("<=".to_string()));
            cs.next();
            cs.next();
        } else if match_string(&cs, "+") {
            tokens.push(Token::Reserved("+".to_string()));
            cs.next();
        } else if match_string(&cs, "-") {
            tokens.push(Token::Reserved("-".to_string()));
            cs.next();
        } else if match_string(&cs, "<") {
            tokens.push(Token::Reserved("<".to_string()));
            cs.next();
        } else if match_string(&cs, ">") {
            tokens.push(Token::Reserved(">".to_string()));
            cs.next();
        } else if match_string(&cs, "(") {
            tokens.push(Token::Reserved("(".to_string()));
            cs.next();
        } else if match_string(&cs, ")") {
            tokens.push(Token::Reserved(")".to_string()));
            cs.next();
        } else if match_string(&cs, ";") {
            tokens.push(Token::Reserved(";".to_string()));
            cs.next();
        } else if match_string(&cs, "=") {
            tokens.push(Token::Reserved("=".to_string()));
            cs.next();
        } else if match_string(&cs, "return") {
            tokens.push(Token::Return);
            for _ in 0..6 {
                cs.next();
            }
        } else if let Some(name) = match_variable_string(&mut cs) {
            let n = name.len();
            tokens.push(Token::Ident(name));
            for _ in 0..n {
                cs.next();
            }
        } else {
            match c {
                ' ' => {
                    cs.next();
                }
                c if c.is_ascii_digit() => {
                    let num = parse_number(&mut cs)
                        .map_err(|e| TokenizeError::new(e.message, 0, input.to_string(), pos))?;
                    tokens.push(Token::Num(num));
                }
                _ => {
                    return Err(TokenizeError::new(
                        "トークナイズ出来ません".to_string(),
                        0,
                        input.to_string(),
                        pos,
                    ))
                }
            }
        }
    }

    tokens.push(Token::Eof);

    Ok(tokens)
}
