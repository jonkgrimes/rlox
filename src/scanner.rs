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

    Token::new(
      TokenKind::Error,
      self.start,
      self.current - self.start,
      self.line,
    )
  }

  pub fn at_end(&mut self) -> bool {
    self.current >= self.source.len()
  }
}
