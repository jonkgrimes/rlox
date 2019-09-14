use std::str::FromStr;

use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::scanner::Scanner;
use crate::token::{Token, TokenKind};

pub struct CompilerError(String);

#[derive(PartialEq, PartialOrd)]
enum Precedence {
  None,
  Assignment,
  Or,
  And,
  Equality,
  Comparison,
  Term,
  Factor,
  Unary,
  Call,
  Primary,
}

type ParseFn = fn(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk);

struct ParseRule {
  pub prefix: Option<ParseFn>,
  pub infix: Option<ParseFn>,
  pub precedence: Precedence,
}

struct Compiler<'a> {
  source: &'a str,
  current: Option<Token>,
  previous: Option<Token>,
  had_error: bool,
}

pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
  let mut compiler = Compiler::new(source);
  compiler.compile(source, chunk)
}

impl<'a> Compiler<'a> {
  fn new(source: &str) -> Compiler {
    Compiler {
      source,
      current: None,
      previous: None,
      had_error: false,
    }
  }

  fn compile(&mut self, source: &str, chunk: &mut Chunk) -> bool {
    let mut scanner = Scanner::new(source);
    self.advance(&mut scanner);
    self.expression(&mut scanner, chunk);
    self.consume(
      &mut scanner,
      TokenKind::Eof,
      "Expected the end of an expression.",
    );
    // emit return
    chunk.write_chunk(OpCode::Return, self.current.as_ref().unwrap().line as u32);
    if self.had_error {
      chunk.disassemble("Total Chunk")
    }
    !self.had_error
  }

  fn advance(&mut self, scanner: &mut Scanner) {
    self.previous = self.current.take();

    loop {
      self.current = Some(scanner.scan_token());

      if let Some(token) = &self.current {
        match &token.kind {
          TokenKind::Error(err_msg) => {
            self.error_at_current(err_msg);
            self.had_error = true;
          }
          _ => break,
        }
      }
    }
  }

  fn consume(&mut self, scanner: &mut Scanner, kind: TokenKind, message: &str) {
    if let Some(current) = &self.current {
      if current.kind == kind {
        self.advance(scanner);
      }
    }
  }

  fn error_at(&self, token: Token, message: &str) {
    print!("[line {}] Error", token.line);

    match token.kind {
      TokenKind::Eof => print!(" at end of line."),
      TokenKind::Error(_) => (),
      _ => {
        let range = token.start..(token.start + token.length);
        print!(" at '{}'", self.source.get(range).unwrap());
      }
    }
  }

  fn error_at_current(&self, message: &str) {
    let current = self.current.clone().unwrap();
    self.error_at(current, message);
  }

  fn parse_precedence(&mut self, precedence: Precedence, scanner: &mut Scanner, chunk: &mut Chunk) {
    self.advance(scanner);
    let parse_rule = self.get_rule(&self.previous.as_ref().unwrap().kind.clone());
    if let Some(prefix_fn) = parse_rule.prefix {
      prefix_fn(self, scanner, chunk);
    } else {
      self.error_at_current("Expect expression.");
      return ();
    }

    while precedence
      <= self
        .get_rule(&self.current.as_ref().unwrap().kind.clone())
        .precedence
    {
      self.advance(scanner);
      let infix_fn = self
        .get_rule(&self.previous.as_ref().unwrap().kind.clone())
        .infix
        .unwrap();
      infix_fn(self, scanner, chunk);
    }
  }

  fn get_rule(&mut self, operator: &TokenKind) -> ParseRule {
    match operator {
      TokenKind::LeftParen => ParseRule {
        prefix: Some(Compiler::grouping),
        infix: None,
        precedence: Precedence::None,
      },
      TokenKind::Minus => ParseRule {
        prefix: Some(Compiler::unary),
        infix: Some(Compiler::binary),
        precedence: Precedence::Term,
      },
      TokenKind::Plus => ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Precedence::Term,
      },
      TokenKind::Slash => ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Precedence::Factor,
      },
      TokenKind::Star => ParseRule {
        prefix: None,
        infix: Some(Compiler::binary),
        precedence: Precedence::Factor,
      },
      TokenKind::Number => ParseRule {
        prefix: Some(Compiler::number),
        infix: None,
        precedence: Precedence::None,
      },
      _ => ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
      },
    }
  }

  fn expression(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    self.parse_precedence(Precedence::Assignment, scanner, chunk);
  }

  fn number(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk) {
    if let Some(token) = &compiler.previous {
      let source = compiler
        .source
        .get(token.start..(token.start + token.length));
      match source {
        Some(code) => {
          let value = f32::from_str(code).ok();
          if let Some(constant) = value {
            let index = chunk.add_constant(constant);
            chunk.write_chunk(OpCode::Constant(index), scanner.line() as u32);
          }
        }
        None => (),
      }
    }
  }

  fn grouping(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk) {
    compiler.expression(scanner, chunk);
    compiler.consume(
      scanner,
      TokenKind::RightParen,
      "Expect a ')' after expression.",
    );
  }

  fn unary(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk) {
    compiler.parse_precedence(Precedence::Unary, scanner, chunk);
    compiler.expression(scanner, chunk);

    match compiler.previous.as_ref().unwrap().kind {
      TokenKind::Minus => {
        chunk.write_chunk(OpCode::Negate, scanner.line() as u32);
      }
      _ => (),
    }
  }

  fn binary(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk) {
    let operator = compiler.previous.as_ref().unwrap().kind.clone();
    let rule = compiler.get_rule(&operator);
    compiler.parse_precedence(rule.precedence, scanner, chunk);

    match operator {
      TokenKind::Plus => chunk.write_chunk(OpCode::Add, scanner.line() as u32),
      TokenKind::Minus => chunk.write_chunk(OpCode::Subtract, scanner.line() as u32),
      TokenKind::Star => chunk.write_chunk(OpCode::Multiply, scanner.line() as u32),
      TokenKind::Slash => chunk.write_chunk(OpCode::Divide, scanner.line() as u32),
      _ => (),
    }
  }
}
