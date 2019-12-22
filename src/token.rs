#[derive(Clone, Debug, PartialEq)]
pub struct Token {
  pub kind: TokenKind,
  pub start: usize,
  pub length: usize,
  pub line: i32,
}

impl Token {
  pub fn new(kind: TokenKind, start: usize, length: usize, line: i32) -> Token {
    Token {
      kind,
      start,
      length,
      line,
    }
  }

  pub fn error(message: &str, start: usize, current: usize, line: i32) -> Token {
    Token::new(
      TokenKind::Error(message.to_string()),
      start,
      current - start,
      line,
    )
  }
}

#[derive(Clone, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_token_equality() {
    let a = Token::new(TokenKind::Var, 1, 1, 1);
    let b = Token::new(TokenKind::Var, 1, 1, 1);

    assert!(a == b);
  }

  #[test]
  fn test_token_inequality() {
    let a = Token::new(TokenKind::Var, 1, 1, 1);
    let b = Token::new(TokenKind::Var, 2, 2, 1);

    assert!(a != b);
  }
}
