use std::env::args;
use std::iter::Peekable;
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

struct TokenStream {
    inner: Peekable<IntoIter<Token>>,
}

impl TokenStream {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            inner: tokens.into_iter().peekable(),
        }
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

    fn expect(&mut self, op: &str) {
        match self.inner.next().unwrap() {
            Token::Reserved(s) if s.as_str() == op => {}
            _ => {
                panic!("'{:?}'ではありません", op);
            }
        }
    }

    fn expect_number(&mut self) -> i64 {
        match self.inner.next().unwrap() {
            Token::Num(n) => n,
            _ => {
                panic!("数値ではありません");
            }
        }
    }

    fn at_eof(&mut self) -> bool {
        match self.inner.peek().unwrap() {
            Token::Eof => true,
            _ => false,
        }
    }
}

fn parse_number(iter: &mut Peekable<impl Iterator<Item = char>>) -> i64 {
    let mut s = String::new();

    while let Some(c) = iter.next_if(|c| c.is_ascii_digit()) {
        s.push(c);
    }
    i64::from_str(s.as_str()).unwrap()
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut cs = input.chars().peekable();
    let mut tokens = vec![];

    while let Some(c) = cs.peek() {
        match c {
            ' ' => {
                cs.next();
            }
            '+' | '-' => {
                tokens.push(Token::Reserved(c.to_string()));
                cs.next();
            }
            c if c.is_ascii_digit() => {
                let num = parse_number(&mut cs);
                tokens.push(Token::Num(num));
            }
            _ => {
                panic!("トークナイズ出来ません");
            }
        }
    }

    tokens.push(Token::Eof);

    tokens
}

fn main() {
    let argv: Vec<_> = args().collect();
    assert_eq!(args().len(), 2);

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let p = argv[1].as_str();
    let mut tokens = tokenize(p);

    let mut token_stream = TokenStream::new(tokens);
    let first = token_stream.expect_number();
    println!("  mov rax, {}", first);

    while !token_stream.at_eof() {
        if token_stream.consume("+") {
            println!("  add rax, {}", token_stream.expect_number());
            continue;
        }

        token_stream.expect("-");
        println!("  sub rax, {}", token_stream.expect_number());
    }

    println!("  ret\n");
}
