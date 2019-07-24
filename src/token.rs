#[derive(Debug)]
pub struct Token {
  pub kind: TokenKind,
  pub start: usize,
  pub length: usize,
  pub line: i32,
}

impl Token {
  pub fn new(kind: TokenKind, start: usize, length: usize, line: i32) -> Token {
    Token {
      kind, start, length, line
    }
  }
}

#[derive(Debug,  PartialEq)]
pub enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error(String),
    Eof,
}