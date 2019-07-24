use crate::token::{Token, TokenKind};

pub struct Scanner<'a> {
  source: &'a str,
  start: usize,
  current: usize,
  line: i32,
}

impl<'a> Scanner<'a> {
  pub fn new(source: &'a str) -> Scanner {
    Scanner {
      source,
      start: 0,
      current: 0,
      line: 1,
    }
  }

  pub fn scan_token(&mut self) -> Token {
    self.start = self.current;

    if self.at_end() {
      return Token::new(
        TokenKind::Eof,
        self.start,
        self.current - self.start,
        self.line,
      );
    }

    match self.advance() {
      Some("(") => self.make_token(TokenKind::LeftParen),
      Some(")") => self.make_token(TokenKind::RightParen),
      _ => {
        Token::new(
          TokenKind::Error("Unexpected token.".to_string()),
          self.start,
          self.current - self.start,
          self.line,
        )
      }
    }
  }

  fn advance(&mut self) -> Option<&str> {
    self.current += 1;
    self.source.get((self.current - 1)..self.current)
  }

  fn make_token(&self, kind: TokenKind) -> Token {
    Token {
      kind,
      start: self.current,
      length: self.current - self.start,
      line: self.line,
    }
  }

  pub fn at_end(&mut self) -> bool {
    self.current >= self.source.len()
  }
}
