use std::collections::HashSet;
use std::str::FromStr;

use crate::chunk::Chunk;
use crate::object::Object;
use crate::op_code::OpCode;
use crate::scanner::Scanner;
use crate::token::{Token, TokenKind};
use crate::value::Value;

pub struct CompilerError(String);

#[derive(Debug, PartialEq, PartialOrd)]
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

type ParseFn =
  fn(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk, can_assign: bool);

struct ParseRule {
  pub prefix: Option<ParseFn>,
  pub infix: Option<ParseFn>,
  pub precedence: Precedence,
}

impl ParseRule {
  fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
    ParseRule {
      prefix,
      infix,
      precedence,
    }
  }
}

struct Compiler<'a> {
  source: &'a str,
  strings: &'a mut HashSet<String>,
  current: Option<Token>,
  previous: Option<Token>,
  had_error: bool,
  locals: Vec<Local>,
  local_count: u32,
  scope_depth: u32,
}

#[derive(Debug)]
struct Local {
  name: Token,
  depth: u32,
}

pub fn compile(source: &str, chunk: &mut Chunk, strings: &mut HashSet<String>) -> bool {
  let mut compiler = Compiler::new(source, strings);
  compiler.compile(source, chunk)
}

impl<'a> Compiler<'a> {
  fn new(source: &'a str, strings: &'a mut HashSet<String>) -> Compiler<'a> {
    Compiler {
      source,
      strings,
      current: None,
      previous: None,
      had_error: false,
      locals: Vec::new(),
      local_count: 0,
      scope_depth: 0,
    }
  }

  fn compile(&mut self, source: &str, chunk: &mut Chunk) -> bool {
    let mut scanner = Scanner::new(source);
    self.advance(&mut scanner);

    loop {
      if self.matches(TokenKind::Eof, &mut scanner) {
        break;
      }
      self.declaration(&mut scanner, chunk);
    }

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
      TokenKind::Eof => println!(" at end of line."),
      TokenKind::Error(error) => println!(": {}", error),
      _ => {
        let range = token.start..(token.start + token.length);
        println!(" at '{}'", self.source.get(range).unwrap());
      }
    }
  }

  fn error_at_current(&self, message: &str) {
    let current = self.current.clone().unwrap();
    self.error_at(current, message);
  }

  fn parse_precedence(&mut self, precedence: Precedence, scanner: &mut Scanner, chunk: &mut Chunk) {
    self.advance(scanner);
    let can_assign = precedence <= Precedence::Assignment;

    let parse_rule = self.get_rule(&self.previous.as_ref().unwrap().kind.clone());
    if let Some(prefix_fn) = parse_rule.prefix {
      prefix_fn(self, scanner, chunk, can_assign);
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
      infix_fn(self, scanner, chunk, can_assign);
    }

    if can_assign && self.matches(TokenKind::Equal, scanner) {
      self.error_at_current("Invalid assignment target.");
      self.expression(scanner, chunk);
    }
  }

  fn get_rule(&mut self, operator: &TokenKind) -> ParseRule {
    match operator {
      TokenKind::LeftParen => ParseRule::new(Some(Compiler::grouping), None, Precedence::None),
      TokenKind::Minus => ParseRule::new(
        Some(Compiler::unary),
        Some(Compiler::binary),
        Precedence::Term,
      ),
      TokenKind::Bang => ParseRule::new(Some(Compiler::unary), None, Precedence::None),
      TokenKind::False => ParseRule::new(Some(Compiler::literal), None, Precedence::None),
      TokenKind::True => ParseRule::new(Some(Compiler::literal), None, Precedence::None),
      TokenKind::Nil => ParseRule::new(Some(Compiler::literal), None, Precedence::None),
      TokenKind::Or => ParseRule::new(None, Some(Compiler::or), Precedence::Or),
      TokenKind::Plus => ParseRule::new(None, Some(Compiler::binary), Precedence::Term),
      TokenKind::Slash => ParseRule::new(None, Some(Compiler::binary), Precedence::Factor),
      TokenKind::Star => ParseRule::new(None, Some(Compiler::binary), Precedence::Factor),
      TokenKind::Number => ParseRule::new(Some(Compiler::number), None, Precedence::None),
      TokenKind::And => ParseRule::new(None, Some(Compiler::and), Precedence::And),
      TokenKind::BangEqual => ParseRule::new(None, Some(Compiler::binary), Precedence::Equality),
      TokenKind::EqualEqual => ParseRule::new(None, Some(Compiler::binary), Precedence::Equality),
      TokenKind::Greater => ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison),
      TokenKind::GreaterEqual => {
        ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison)
      }
      TokenKind::Less => ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison),
      TokenKind::LessEqual => ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison),
      TokenKind::Identifier => ParseRule::new(Some(Compiler::variable), None, Precedence::None),
      TokenKind::String => ParseRule::new(Some(Compiler::string), None, Precedence::None),
      _ => ParseRule::new(None, None, Precedence::None),
    }
  }

  fn matches(&mut self, kind: TokenKind, scanner: &mut Scanner) -> bool {
    if !self.current.as_ref().map_or(false, |c| c.kind == kind) {
      return false;
    }
    self.advance(scanner);
    true
  }

  fn declaration(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    if self.matches(TokenKind::Var, scanner) {
      self.var_declaration(scanner, chunk);
    } else {
      self.statement(scanner, chunk)
    }
  }

  fn var_declaration(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    let global = self.parse_variable("Expect variable name", scanner, chunk);

    if self.matches(TokenKind::Equal, scanner) {
      self.expression(scanner, chunk);
    } else {
      chunk.write_chunk(OpCode::Nil, self.current.as_ref().unwrap().line as u32);
    }

    self.consume(
      scanner,
      TokenKind::Semicolon,
      "Expect ';' after variable declaration.",
    );

    self.define_variable(global, chunk);
  }

  fn parse_variable(&mut self, error: &str, scanner: &mut Scanner, chunk: &mut Chunk) -> usize {
    self.consume(scanner, TokenKind::Identifier, error);

    self.declare_variable();
    if self.scope_depth > 0 {
      return 0;
    }

    self.identifier_constant(self.previous.as_ref().unwrap(), chunk)
  }

  fn declare_variable(&mut self) {
    if self.scope_depth == 0 {
      return;
    }

    let name = self.previous.as_ref().unwrap().clone();

    for local in &self.locals {
      if local.depth < self.scope_depth {
        break;
      }

      if local.name == name {
        self.error_at_current("Variable with this name already declared in this scope.")
      }
    }

    self.add_local(name);
  }

  fn add_local(&mut self, name: Token) {
    self.locals.push(Local {
      name,
      depth: self.scope_depth,
    })
  }

  fn define_variable(&self, index: usize, chunk: &mut Chunk) {
    if self.scope_depth > 0 {
      return;
    }

    chunk.write_chunk(
      OpCode::DefineGlobal(index),
      self.current.as_ref().unwrap().line as u32,
    );
  }

  fn identifier_constant(&self, token: &Token, chunk: &mut Chunk) -> usize {
    let source = self.source.get((token.start)..(token.start + token.length));
    let identifier = Box::new(Object::String(String::from(source.unwrap())));
    let index = chunk.add_constant(Value::String(Box::into_raw(identifier)));
    index
  }

  fn statement(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    if self.matches(TokenKind::Print, scanner) {
      self.print_statement(scanner, chunk);
    } else if self.matches(TokenKind::If, scanner) {
      self.if_statement(scanner, chunk);
    } else if self.matches(TokenKind::LeftBrace, scanner) {
      self.begin_scope(scanner, chunk);
      self.block(scanner, chunk);
      self.end_scope(scanner, chunk);
    } else {
      self.expression_statement(scanner, chunk);
    }
  }

  fn print_statement(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    self.expression(scanner, chunk);
    self.consume(scanner, TokenKind::Semicolon, "Expect ';' after value.");
    chunk.write_chunk(OpCode::Print, self.current.as_ref().unwrap().line as u32);
  }

  fn if_statement(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    self.consume(scanner, TokenKind::LeftParen, "Expect '(' after 'if'");
    self.expression(scanner, chunk);
    self.consume(
      scanner,
      TokenKind::RightParen,
      "Expect ')' after condition.",
    );

    let then_jmp = self.emit_jump(OpCode::JumpIfFalse(0), chunk);
    let line = self.current.as_ref().unwrap().line as u32;
    chunk.write_chunk(OpCode::Pop, line);
    self.statement(scanner, chunk);

    let else_jmp = self.emit_jump(OpCode::Jump(0), chunk);

    self.patch_jump(then_jmp, chunk);
    let line = self.current.as_ref().unwrap().line as u32;
    chunk.write_chunk(OpCode::Pop, line);

    if self.matches(TokenKind::Else, scanner) {
      self.statement(scanner, chunk);
    }
    self.patch_jump(else_jmp, chunk);
  }

  fn emit_jump(&mut self, op_code: OpCode, chunk: &mut Chunk) -> usize {
    let line = self.current.as_ref().unwrap().line as u32;
    chunk.write_chunk(op_code, line);
    chunk.code.len() - 1
  }

  fn patch_jump(&mut self, jmp: usize, chunk: &mut Chunk) {
    let offset = chunk.code.len() - jmp - 1;
    match chunk.code.get(jmp) {
      Some(OpCode::Jump(_)) => chunk.code[jmp] = OpCode::Jump(offset),
      Some(OpCode::JumpIfFalse(_)) => {
        chunk.code[jmp] = OpCode::JumpIfFalse(offset);
      }
      _ => {}
    }
  }

  fn expression_statement(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    self.expression(scanner, chunk);
    self.consume(
      scanner,
      TokenKind::Semicolon,
      "Expect ';' after expression.",
    );
    chunk.write_chunk(OpCode::Pop, self.current.as_ref().unwrap().line as u32);
  }

  fn begin_scope(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    self.scope_depth += 1;
  }

  fn block(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    loop {
      if self.matches(TokenKind::RightBrace, scanner) || self.matches(TokenKind::Eof, scanner) {
        break;
      }

      self.declaration(scanner, chunk)
    }
  }

  fn end_scope(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    self.scope_depth -= 1;

    loop {
      if !(self.local_count > 0) || self.locals.last().unwrap().depth <= self.scope_depth {
        break;
      }

      chunk.write_chunk(OpCode::Pop, scanner.line() as u32);
      self.local_count -= 1;
    }
  }

  fn expression(&mut self, scanner: &mut Scanner, chunk: &mut Chunk) {
    self.parse_precedence(Precedence::Assignment, scanner, chunk);
  }

  fn number(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk, _can_assgin: bool) {
    if let Some(token) = &compiler.previous {
      let source = compiler
        .source
        .get(token.start..(token.start + token.length));
      match source {
        Some(code) => {
          let value = f32::from_str(code).ok();
          if let Some(constant) = value {
            let index = chunk.add_constant(Value::Number(constant));
            chunk.write_chunk(OpCode::Constant(index), scanner.line() as u32);
          }
        }
        None => (),
      }
    }
  }

  fn string(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk, _can_assign: bool) {
    if let Some(token) = &compiler.previous {
      let source = compiler
        .source
        .get((token.start + 1)..(token.start + token.length - 1));
      match source {
        Some(string) => {
          let value = if let Some(existing_string) = compiler.strings.get(string) {
            Box::new(Object::String(existing_string.to_string()))
          } else {
            Box::new(Object::String(String::from(string)))
          };
          let index = chunk.add_constant(Value::String(Box::into_raw(value)));
          chunk.write_chunk(OpCode::Constant(index), scanner.line() as u32);
        }
        None => (),
      }
    }
  }

  fn grouping(
    compiler: &mut Compiler,
    scanner: &mut Scanner,
    chunk: &mut Chunk,
    _can_assgin: bool,
  ) {
    compiler.expression(scanner, chunk);
    compiler.consume(
      scanner,
      TokenKind::RightParen,
      "Expect a ')' after expression.",
    );
  }

  fn unary(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk, _can_assign: bool) {
    let operator = compiler.previous.as_ref().unwrap().kind.clone();

    compiler.parse_precedence(Precedence::Unary, scanner, chunk);

    match operator {
      TokenKind::Minus => {
        chunk.write_chunk(OpCode::Negate, scanner.line() as u32);
      }
      TokenKind::Bang => {
        chunk.write_chunk(OpCode::Not, scanner.line() as u32);
      }
      _ => (),
    }
  }

  fn binary(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk, _can_assign: bool) {
    let operator = compiler.previous.as_ref().unwrap().kind.clone();
    let rule = compiler.get_rule(&operator);
    compiler.parse_precedence(rule.precedence, scanner, chunk);

    let line = scanner.line() as u32;

    match operator {
      TokenKind::Plus => chunk.write_chunk(OpCode::Add, line),
      TokenKind::Minus => chunk.write_chunk(OpCode::Subtract, line),
      TokenKind::Star => chunk.write_chunk(OpCode::Multiply, line),
      TokenKind::Slash => chunk.write_chunk(OpCode::Divide, line),
      TokenKind::BangEqual => {
        chunk.write_chunk(OpCode::Equal, line);
        chunk.write_chunk(OpCode::Not, line);
      }
      TokenKind::EqualEqual => chunk.write_chunk(OpCode::Equal, line),
      TokenKind::Greater => chunk.write_chunk(OpCode::Greater, line),
      TokenKind::GreaterEqual => {
        chunk.write_chunk(OpCode::Greater, line);
        chunk.write_chunk(OpCode::Not, line);
      }
      TokenKind::Less => chunk.write_chunk(OpCode::Less, line),
      TokenKind::LessEqual => {
        chunk.write_chunk(OpCode::Less, line);
        chunk.write_chunk(OpCode::Not, line);
      }
      _ => (),
    }
  }

  fn and(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk, _can_assign: bool) {
    let end_jump = compiler.emit_jump(OpCode::JumpIfFalse(0), chunk);

    chunk.write_chunk(OpCode::Pop, scanner.line() as u32);

    compiler.parse_precedence(Precedence::And, scanner, chunk);

    compiler.patch_jump(end_jump, chunk);
  }

  fn or(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk, _can_assign: bool) {
    let else_jump = compiler.emit_jump(OpCode::JumpIfFalse(0), chunk);
    let end_jump = compiler.emit_jump(OpCode::Jump(0), chunk);

    compiler.patch_jump(else_jump, chunk);
    chunk.write_chunk(OpCode::Pop, scanner.line() as u32);

    compiler.parse_precedence(Precedence::Or, scanner, chunk);
    compiler.patch_jump(end_jump, chunk);
  }

  fn literal(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk, _can_assign: bool) {
    let operator = compiler.previous.as_ref().unwrap().kind.clone();
    match operator {
      TokenKind::Nil => chunk.write_chunk(OpCode::Nil, scanner.line() as u32),
      TokenKind::True => chunk.write_chunk(OpCode::True, scanner.line() as u32),
      TokenKind::False => chunk.write_chunk(OpCode::False, scanner.line() as u32),
      _ => (),
    }
  }

  fn variable(compiler: &mut Compiler, scanner: &mut Scanner, chunk: &mut Chunk, can_assign: bool) {
    compiler.named_variable(scanner, chunk, can_assign);
  }

  fn named_variable(&mut self, scanner: &mut Scanner, chunk: &mut Chunk, can_assign: bool) {
    let token = self.previous.as_ref().unwrap();
    let get_op;
    let set_op;

    if let Some(arg) = self.resolve_local(token) {
      get_op = OpCode::GetLocal(arg as usize);
      set_op = OpCode::SetLocal(arg as usize);
    } else {
      let index = self.identifier_constant(token, chunk);
      get_op = OpCode::GetGlobal(index);
      set_op = OpCode::SetGlobal(index);
    }
    let line = scanner.line() as u32;

    if can_assign && self.matches(TokenKind::Equal, scanner) {
      self.expression(scanner, chunk);
      chunk.write_chunk(set_op, line);
    } else {
      chunk.write_chunk(get_op, line);
    }
  }

  fn resolve_local(&self, name: &Token) -> Option<usize> {
    for (i, local) in self.locals.iter().enumerate().rev() {
      if self.identifiers_equal(&local.name, name) {
        return Some(i);
      }
    }
    None
  }

  fn identifiers_equal(&self, lhs: &Token, rhs: &Token) -> bool {
    if lhs.length != rhs.length {
      return false;
    }
    let lhs_range = lhs.start..(lhs.start + lhs.length);
    let rhs_range = rhs.start..(rhs.start + rhs.length);
    if self.source[lhs_range] == self.source[rhs_range] {
      return true;
    }
    false
  }
}
