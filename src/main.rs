use std::env::args;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::num::ParseIntError;
use std::slice::Iter;
use std::str::FromStr;
use std::vec::IntoIter;

#[derive(Debug)]
enum Token {
    Reserved(String),
    Num(i64),
    Eof,
}

impl Token {
    fn reserved(s: &str) -> Self {
        Token::Reserved(s.to_string())
    }

    fn as_reserved(&self) -> Option<&String> {
        match self {
            Token::Reserved(s) => Some(s),
            _ => None,
        }
    }

    fn as_num(&self) -> Option<i64> {
        match self {
            Token::Num(n) => Some(*n),
            _ => None,
        }
    }

    fn as_eof(&self) -> bool {
        match self {
            Token::Eof => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
enum ParseError {
    ExpectReserved(String),
    ExpectNumber,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

type ParseResult<T> = std::result::Result<T, ParseError>;

struct TokenStream {
    inner: Peekable<IntoIter<Token>>,
}

impl TokenStream {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            inner: tokens.into_iter().peekable(),
        }
    }

    fn primary(&mut self) -> ParseResult<Node> {
        if self.consume("(") {
            let node = self.expr()?;
            self.expect(")")?;
            Ok(node)
        } else {
            let number = self.expect_number()?;
            Ok(Node::Num(number))
        }
    }

    fn unary(&mut self) -> ParseResult<Node> {
        if self.consume("+") {
            self.primary()
        } else if self.consume("-") {
            Ok(Node::new_op2(
                Operator2::Sub,
                Box::new(Node::Num(0)),
                Box::new(self.primary()?),
            ))
        } else {
            self.primary()
        }
    }

    fn mul(&mut self) -> ParseResult<Node> {
        let mut node = self.unary()?;

        loop {
            node = if self.consume("*") {
                Node::new_op2(Operator2::Mul, Box::new(node), Box::new(self.unary()?))
            } else if self.consume("/") {
                Node::new_op2(Operator2::Div, Box::new(node), Box::new(self.unary()?))
            } else {
                break;
            }
        }

        Ok(node)
    }

    fn expr(&mut self) -> ParseResult<Node> {
        let mut node = self.mul()?;

        loop {
            if self.consume("+") {
                let right = self.mul()?;
                node = Node::new_op2(Operator2::Add, Box::new(node), Box::new(right))
            } else if self.consume("-") {
                let right = self.mul()?;
                node = Node::new_op2(Operator2::Sub, Box::new(node), Box::new(right))
            } else {
                break;
            }
        }

        Ok(node)
    }

    fn consume(&mut self, op: &str) -> bool {
        match self.inner.peek().unwrap() {
            Token::Reserved(s) if s.as_str() == op => {
                self.inner.next().unwrap();
                true
            }
            _ => false,
        }
    }

    fn expect(&mut self, op: &str) -> ParseResult<()> {
        match self.inner.next().unwrap() {
            Token::Reserved(s) if s.as_str() == op => Ok(()),
            _ => Err(ParseError::ExpectReserved(op.to_string())),
        }
    }

    fn expect_number(&mut self) -> ParseResult<i64> {
        match self.inner.next().unwrap() {
            Token::Num(n) => Ok(n),
            _ => return Err(ParseError::ExpectNumber),
        }
    }

    fn at_eof(&mut self) -> bool {
        match self.inner.peek().unwrap() {
            Token::Eof => true,
            _ => false,
        }
    }
}

fn parse_number(
    iter: &mut Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<i64, GeneralError> {
    let mut s = String::new();

    while let Some((pos, c)) = iter.next_if(|(pos, c)| c.is_ascii_digit()) {
        s.push(c);
    }
    i64::from_str(s.as_str()).map_err(|_| GeneralError::new("整数がパースできません".to_string()))
}

#[derive(Debug)]
enum Operator2 {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
enum Node {
    Operator2 {
        op: Operator2,
        left: Box<Self>,
        right: Box<Self>,
    },
    Num(i64),
}

impl Node {
    pub fn new_op2(op: Operator2, left: Box<Self>, right: Box<Self>) -> Self {
        Self::Operator2 { op, left, right }
    }
}

impl Node {
    fn gen(&self) {
        match self {
            Node::Num(n) => {
                println!("  push {}", n);
            }
            Node::Operator2 { op, left, right } => {
                left.gen();
                right.gen();
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
                }

                println!("  push rax");
            }
        }
    }
}

#[derive(Debug)]
struct GeneralError {
    message: String,
}

impl GeneralError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Display for GeneralError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for GeneralError {}

#[derive(Debug)]
struct TokenizeError {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for TokenizeError {}

type TokenizeResult<T> = std::result::Result<T, TokenizeError>;

fn tokenize(input: &str) -> TokenizeResult<Vec<Token>> {
    let mut cs = input.chars().enumerate().peekable();
    let mut tokens = vec![];

    while let Some((pos, c)) = cs.peek().cloned() {
        match c {
            ' ' => {
                cs.next();
            }
            '+' | '-' => {
                tokens.push(Token::Reserved(c.to_string()));
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

    tokens.push(Token::Eof);

    Ok(tokens)
}

fn main() {
    let argv: Vec<_> = args().collect();
    assert_eq!(args().len(), 2);

    let p = argv[1].as_str();
    let mut tokens = tokenize(p).unwrap();

    let mut token_stream = TokenStream::new(tokens);
    let node = token_stream.expr().unwrap();

    println!(".intel_syntax noprefix");
    println!(".globl _main");
    println!("_main:");

    node.gen();

    println!("  pop rax");
    println!("  ret");
}
