use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::FromStr;

#[derive(Debug)]
pub enum Token {
    Reserved(String),
    Ident(String),
    Num(i64),
    Return,
    If,
    Else,
    For,
    While,
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
        } else if match_string(&cs, "*") {
            tokens.push(Token::Reserved("*".to_string()));
            cs.next();
        } else if match_string(&cs, "/") {
            tokens.push(Token::Reserved("/".to_string()));
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
        } else if match_string(&cs, "{") {
            tokens.push(Token::Reserved("{".to_string()));
            cs.next();
        } else if match_string(&cs, "}") {
            tokens.push(Token::Reserved("}".to_string()));
            cs.next();
        } else if match_string(&cs, ",") {
            tokens.push(Token::Reserved(",".to_string()));
            cs.next();
        } else if match_string(&cs, "&") {
            tokens.push(Token::Reserved("&".to_string()));
            cs.next();
        } else if match_string(&cs, "return") {
            tokens.push(Token::Return);
            for _ in 0..6 {
                cs.next();
            }
        } else if match_string(&cs, "if") {
            tokens.push(Token::If);
            for _ in 0..2 {
                cs.next();
            }
        } else if match_string(&cs, "else") {
            tokens.push(Token::Else);
            for _ in 0..4 {
                cs.next();
            }
        } else if match_string(&cs, "for") {
            tokens.push(Token::For);
            for _ in 0..3 {
                cs.next();
            }
        } else if match_string(&cs, "while") {
            tokens.push(Token::While);
            for _ in 0..5 {
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

pub fn parse_number(
    iter: &mut Peekable<impl Iterator<Item = (usize, char)>>,
) -> Result<i64, GeneralError> {
    let mut s = String::new();

    while let Some((_pos, c)) = iter.next_if(|(_pos, c)| c.is_ascii_digit()) {
        s.push(c);
    }
    i64::from_str(s.as_str()).map_err(|_| GeneralError::new("整数がパースできません".to_string()))
}
