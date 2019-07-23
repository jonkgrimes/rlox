use crate::scanner::Scanner;
use crate::token::{Token, TokenKind};

pub fn compile(source: &str) {
  let mut scanner = Scanner::new(source);
  let mut line = -1i32;
  loop { 
    let token = scanner.scan_token();
    if token.line != line {
      println!("{}", token.line);
      line = token.line;
    } else {
      println!("   | ")
    }
    println!("{:?}, {}", token.kind, source.get(token.start..(token.start + token.length)).unwrap());

    if token.kind == TokenKind::Eof  {
      break;
    }
  }
}