use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use crate::core::{Closure, Function, FunctionType, Object, Value};
use crate::vm::OpCode;
use crate::scanner::{Scanner, Token, TokenKind};

#[derive(Debug)]
pub struct CompilerError(String);

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

type ParseFn = fn(compiler: &mut Compiler, scanner: &mut Scanner, can_assign: bool);

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
    states: Vec<CompilerState>,
    upvalues: Vec<Upvalue>,
}


#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Upvalue {
    pub local: bool,
    pub index: usize,
}

impl Upvalue {
    fn new(local: bool, index: usize) -> Self {
        Self { local, index }
    }
}

#[derive(Debug, Clone)]
struct CompilerState {
    function: Function,
    function_type: FunctionType,
    enclosing: Option<usize>,
    scope_depth: usize,
    locals: Vec<Local>,
    local_count: usize,
    upvalue_count: usize,
    upvalues: Vec<Upvalue>
}

#[derive(Debug, Clone)]
struct Local {
    name: Token,
    depth: usize,
}

pub fn compile(
    source: &str,
    function: Function,
    strings: &mut HashSet<String>,
) -> Result<Function, CompilerError> {
    let mut scanner = Scanner::new(source);
    let mut compiler = Compiler::new(source, function, strings);
    compiler.compile(&mut scanner)
}

impl<'a> Compiler<'a> {
    fn new(source: &'a str, function: Function, strings: &'a mut HashSet<String>) -> Compiler<'a> {
        let state: CompilerState = CompilerState {
            function,
            function_type: FunctionType::Script,
            enclosing: None,
            scope_depth: 0,
            local_count: 0,
            locals: Vec::new(),
            upvalue_count: 0,
            upvalues: Vec::with_capacity(u8::MAX as usize)
        };
        Compiler {
            source,
            strings,
            current: None,
            previous: None,
            had_error: false,
            states: vec![state],
            upvalues: Vec::new(),
        }
    }

    fn state(&self) -> &CompilerState {
        self.states.last().unwrap()
    }

    fn current_state_index(&self) -> usize {
        self.states.len() - 1
    }

    fn state_mut(&mut self) -> &mut CompilerState {
        self.states.last_mut().unwrap()
    }

    fn emit_opcode(&mut self, op_code: OpCode) {
        let line = self.current.as_ref().map_or(1, |t| t.line as u32);
        self.state_mut()
            .function
            .chunk
            .write_chunk(op_code, line);
    }

    fn compile(&mut self, scanner: &mut Scanner) -> Result<Function, CompilerError> {
        self.advance(scanner);

        loop {
            if self.matches(TokenKind::Eof, scanner) {
                break;
            }
            self.declaration(scanner);
        }

        // emit return
        self.emit_opcode(OpCode::Nil);
        self.emit_opcode(OpCode::Return);

        if self.had_error {
            self.state()
                .function
                .chunk
                .disassemble("Total Chunk")
        }

        if !self.had_error {
            Ok(self.state().function.clone())
        } else {
            Err(CompilerError(
                "There was an error during compilation".to_string(),
            ))
        }
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
        } else {
            self.error_at_current(message);
        }
    }

    fn error_at(&self, token: Token, message: &str) {
        print!("[line {}] Error", token.line);

        match token.kind {
            TokenKind::Eof => {
                println!(" at end of line: {}", message);
                panic!()
            }
            TokenKind::Error(error) => println!(": {}: {}", error, message),
            _ => {
                let range = token.start..(token.start + token.length);
                println!(": {} at '{}'", message, self.source.get(range).unwrap());
            }
        }
    }

    fn error_at_current(&self, message: &str) {
        let current = self.current.clone().unwrap();
        self.error_at(current, message);
    }

    fn parse_precedence(&mut self, precedence: Precedence, scanner: &mut Scanner) {
        self.advance(scanner);
        let can_assign = precedence <= Precedence::Assignment;

        let parse_rule = self.get_rule(&self.previous.as_ref().unwrap().kind.clone());
        if let Some(prefix_fn) = parse_rule.prefix {
            prefix_fn(self, scanner, can_assign);
        } else {
            self.error_at_current("Expect expression");
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
            infix_fn(self, scanner, can_assign);
        }

        if can_assign && self.matches(TokenKind::Equal, scanner) {
            self.error_at_current("Invalid assignment target.");
            self.expression(scanner);
        }
    }

    fn get_rule(&mut self, operator: &TokenKind) -> ParseRule {
        match operator {
            TokenKind::LeftParen => ParseRule::new(
                Some(Compiler::grouping),
                Some(Compiler::call),
                Precedence::Call,
            ),
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
            TokenKind::BangEqual => {
                ParseRule::new(None, Some(Compiler::binary), Precedence::Equality)
            }
            TokenKind::EqualEqual => {
                ParseRule::new(None, Some(Compiler::binary), Precedence::Equality)
            }
            TokenKind::Greater => {
                ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison)
            }
            TokenKind::GreaterEqual => {
                ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison)
            }
            TokenKind::Less => ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison),
            TokenKind::LessEqual => {
                ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison)
            }
            TokenKind::Identifier => {
                ParseRule::new(Some(Compiler::variable), None, Precedence::None)
            }
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

    fn declaration(&mut self, scanner: &mut Scanner) {
        if self.matches(TokenKind::Fun, scanner) {
            self.fun_declaration(scanner);
        } else if self.matches(TokenKind::Var, scanner) {
            self.var_declaration(scanner);
        } else {
            self.statement(scanner)
        }
    }

    fn fun_declaration(&mut self, scanner: &mut Scanner) {
        let global = self.parse_variable("Expected a function name.", scanner);
        self.mark_initialized();
        self.function(scanner, FunctionType::Function, global);
        self.define_variable(global);
    }

    fn init_state(&mut self, function: Function) {
        let enclosing = Some(self.states.len() - 1);
        let state: CompilerState = CompilerState {
            function,
            function_type: FunctionType::Function,
            enclosing,
            scope_depth: 0,
            local_count: 0,
            locals: Vec::new(),
            upvalue_count: 0,
            upvalues: Vec::new()
        };
        self.states.push(state);
    }

    fn end_state(&mut self) -> Result<Function, CompilerError> {
        self.emit_opcode(OpCode::Nil);
        self.emit_opcode(OpCode::Return);
        let state = self.states.pop().unwrap();
        Ok(state.function)
    }

    fn function(
        &mut self,
        scanner: &mut Scanner,
        function_type: FunctionType,
        constant_index: usize,
    ) {
        let constant = self
            .state()
            .function
            .chunk
            .constants
            .get(constant_index)
            .unwrap();
        let name = match constant {
            Value::Object(object) => {
                match object {
                    Object::String(string) => string,
                    _ => "Undefined"
                }
            }
            _ => "Undefined",
        };
        let function = Function::new(&name, function_type);
        self.init_state(function);
        self.begin_scope();

        // Compile the parameter list
        self.consume(
            scanner,
            TokenKind::LeftParen,
            "Expect '(' after function name.",
        );

        // Parse parameters
        if !self.matches(TokenKind::RightParen, scanner) {
            loop {
                self.state_mut().function.arity += 1;

                if self.state().function.arity > 255 {
                    self.error_at_current("Cannot have more than 255 parameters.");
                }
                let index = self.parse_variable("Expect parameter name.", scanner);
                self.define_variable(index);
                if !self.matches(TokenKind::Comma, scanner) {
                    break;
                }
            }
        }

        self.consume(
            scanner,
            TokenKind::RightParen,
            "Expect ')' after parameters.",
        );
        // Compile the body
        self.consume(
            scanner,
            TokenKind::LeftBrace,
            "Expect '{' before function body.",
        );
        self.block(scanner);

        match self.end_state() {
            Ok(function) => {
                let upvalue_count = function.upvalue_count;
                let closure = Closure::new(function);
                let index = self.add_constant(Value::Object(Object::Closure(closure)));
                self.emit_opcode(OpCode::Closure(index));

                for i in 0..upvalue_count {
                    let upvalue = self.upvalues[i].clone();
                    if upvalue.local {
                        self.emit_opcode(OpCode::LocalValue(upvalue.index))
                    } else {
                        self.emit_opcode(OpCode::Upvalue(upvalue.index))
                    }
                }
            }
            Err(e) => {
                let message = format!("There was a problem compiling the function {}", e);
                self.error_at_current(&message);
            }
        };
    }

    fn var_declaration(&mut self, scanner: &mut Scanner) {
        let global = self.parse_variable("Expect variable name", scanner);

        if self.matches(TokenKind::Equal, scanner) {
            self.expression(scanner);
        } else {
            let line = self.current.as_ref().unwrap().line as u32;
            self.state_mut()
                .function
                .chunk
                .write_chunk(OpCode::Nil, line);
        }

        self.consume(
            scanner,
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }

    fn scope_depth(&self) -> usize {
        self.state().scope_depth
    }

    fn local_count(&self) -> usize {
        self.state().local_count
    }

    fn parse_variable(&mut self, error: &str, scanner: &mut Scanner) -> usize {
        self.consume(scanner, TokenKind::Identifier, error);

        self.declare_variable();

        let identifier = self.previous.as_ref().unwrap().clone();
        self.identifier_constant(&identifier)
    }

    fn declare_variable(&mut self) {
        if self.scope_depth() == 0 {
            return;
        }

        let name = self.previous.as_ref().unwrap().clone();
        let state = self.state();

        for local in &state.locals {
            if local.depth < state.scope_depth {
                break;
            }

            if local.name == name {
                // TODO: Reimplement
                // self.error_at_current("Variable with this name already declared in this scope.")
            }
        }

        self.add_local(name);
    }

    fn add_local(&mut self, name: Token) {
        let scope_depth = self.scope_depth();
        self.state_mut().local_count += 1;
        self.state_mut().locals.push(Local {
            name,
            depth: scope_depth,
        })
    }

    fn define_variable(&mut self, index: usize) {
        if self.scope_depth() > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_opcode(OpCode::DefineGlobal(index));
    }

    fn identifier_constant(&mut self, token: &Token) -> usize {
        let source = self.source.get((token.start)..(token.start + token.length));
        let identifier = String::from(source.unwrap());
        self.add_constant(Value::Object(Object::String(identifier)))
    }

    fn add_constant(&mut self, constant: Value) -> usize {
        self.state_mut()
            .function
            .chunk
            .add_constant(constant)
    }

    fn mark_initialized(&mut self) {
        if self.scope_depth() == 0 {
            return;
        }
        let local_index = self.state().local_count - 1;
        self.state_mut().locals[local_index].depth = self.scope_depth();
    }

    fn statement(&mut self, scanner: &mut Scanner) {
        if self.matches(TokenKind::Print, scanner) {
            self.print_statement(scanner);
        } else if self.matches(TokenKind::For, scanner) {
            self.for_statement(scanner);
        } else if self.matches(TokenKind::If, scanner) {
            self.if_statement(scanner);
        } else if self.matches(TokenKind::While, scanner) {
            self.while_statement(scanner);
        } else if self.matches(TokenKind::Return, scanner) {
            self.return_statement(scanner);
        } else if self.matches(TokenKind::LeftBrace, scanner) {
            self.begin_scope();
            self.block(scanner);
            self.end_scope();
        } else {
            self.expression_statement(scanner);
        }
    }

    fn print_statement(&mut self, scanner: &mut Scanner) {
        self.expression(scanner);
        self.consume(scanner, TokenKind::Semicolon, "Expect ';' after value.");
        self.emit_opcode(OpCode::Print);
    }

    fn if_statement(&mut self, scanner: &mut Scanner) {
        self.consume(scanner, TokenKind::LeftParen, "Expect '(' after 'if'");
        self.expression(scanner);
        self.consume(
            scanner,
            TokenKind::RightParen,
            "Expect ')' after condition.",
        );

        let then_jmp = self.emit_jump(OpCode::JumpIfFalse(0));
        self.emit_opcode(OpCode::Pop);
        self.statement(scanner);

        let else_jmp = self.emit_jump(OpCode::Jump(0));

        self.patch_jump(then_jmp);
        self.emit_opcode(OpCode::Pop);

        if self.matches(TokenKind::Else, scanner) {
            self.statement(scanner);
        }
        self.patch_jump(else_jmp);
    }

    fn return_statement(&mut self, scanner: &mut Scanner) {
        if self.matches(TokenKind::Semicolon, scanner) {
            self.emit_opcode(OpCode::Nil);
            self.emit_opcode(OpCode::Return);
        } else {
            self.expression(scanner);
            self.consume(
                scanner,
                TokenKind::Semicolon,
                "Expect a ';' after a return value.",
            );
            self.emit_opcode(OpCode::Return);
        }
    }

    fn while_statement(&mut self, scanner: &mut Scanner) {
        let loop_start = self.state().function.chunk.code.len();
        self.consume(scanner, TokenKind::LeftParen, "Expect '(' after 'while'.");
        self.expression(scanner);

        self.consume(
            scanner,
            TokenKind::RightParen,
            "Expect ')' after condition.",
        );

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse(0));

        self.emit_opcode(OpCode::Pop);
        self.statement(scanner);

        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_opcode(OpCode::Pop);
    }

    fn for_statement(&mut self, scanner: &mut Scanner) {
        self.begin_scope();

        // Initializer clause
        self.consume(scanner, TokenKind::LeftParen, "Expect '(' after 'for'");
        if self.matches(TokenKind::Semicolon, scanner) {
            // No initializer
        } else if self.matches(TokenKind::Var, scanner) {
            self.var_declaration(scanner);
        } else {
            self.expression_statement(scanner);
        }

        let mut loop_start = self.state().function.chunk.code.len();

        // Condition clause
        let mut exit_jump = None;
        if !self.matches(TokenKind::Semicolon, scanner) {
            self.expression(scanner);
            self.consume(
                scanner,
                TokenKind::Semicolon,
                "Expect ';' after loop condition.",
            );

            exit_jump = Some(self.emit_jump(OpCode::JumpIfFalse(0)));
            self.emit_opcode(OpCode::Pop);
        }

        // Increment clause
        if !self.matches(TokenKind::RightParen, scanner) {
            let body_jump = self.emit_jump(OpCode::Jump(0));

            let inc_start = self.state().function.chunk.code.len();

            self.expression(scanner);
            self.emit_opcode(OpCode::Pop);
            self.consume(
                scanner,
                TokenKind::RightParen,
                "Expect ')' after for clauses.",
            );

            self.emit_loop(loop_start);
            loop_start = inc_start;
            self.patch_jump(body_jump);
        }

        self.statement(scanner);

        self.emit_loop(loop_start);

        if let Some(exit_jump) = exit_jump {
            self.patch_jump(exit_jump);
            self.emit_opcode(OpCode::Pop);
        }

        self.end_scope();
    }

    fn emit_jump(&mut self, op_code: OpCode) -> usize {
        self.emit_opcode(op_code);
        self.state().function.chunk.code.len() - 1
    }

    fn patch_jump(&mut self, jmp: usize) {
        let offset = self.state().function.chunk.code.len() - jmp - 1;
        match self.state().function.chunk.code.get(jmp) {
            Some(OpCode::Jump(_)) => {
                self.state_mut().function.chunk.code[jmp] = OpCode::Jump(offset)
            }
            Some(OpCode::JumpIfFalse(_)) => {
                self.state_mut().function.chunk.code[jmp] = OpCode::JumpIfFalse(offset);
            }
            _ => {}
        }
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let line = self.current.as_ref().unwrap().line as u32;
        let offset = self.state().function.chunk.code.len() - loop_start;
        self.state_mut()
            .function
            .chunk
            .write_chunk(OpCode::Loop(offset), line);
    }

    fn expression_statement(&mut self, scanner: &mut Scanner) {
        self.expression(scanner);
        self.consume(
            scanner,
            TokenKind::Semicolon,
            "Expect ';' after expression.",
        );
        let line = self.current.as_ref().unwrap().line as u32;
        self.state_mut()
            .function
            .chunk
            .write_chunk(OpCode::Pop, line);
    }

    fn begin_scope(&mut self) {
        self.state_mut().scope_depth += 1;
    }

    fn block(&mut self, scanner: &mut Scanner) {
        loop {
            if self.matches(TokenKind::RightBrace, scanner) || self.matches(TokenKind::Eof, scanner)
            {
                break;
            }

            self.declaration(scanner)
        }
    }

    fn end_scope(&mut self) {
        self.state_mut().scope_depth -= 1;

        while self.local_count() > 0
            && self.state().locals[self.local_count() - 1].depth > self.scope_depth()
        {
            self.emit_opcode(OpCode::Pop);
            self.state_mut().local_count -= 1;
        }
    }

    fn expression(&mut self, scanner: &mut Scanner) {
        self.parse_precedence(Precedence::Assignment, scanner);
    }

    fn number(compiler: &mut Compiler, _scanner: &mut Scanner, _can_assgin: bool) {
        if let Some(token) = &compiler.previous {
            let source = compiler
                .source
                .get(token.start..(token.start + token.length));
            match source {
                Some(code) => {
                    let value = f32::from_str(code).ok();
                    if let Some(constant) = value {
                        let index = compiler.add_constant(Value::Number(constant));
                        compiler.emit_opcode(OpCode::Constant(index));
                    }
                }
                None => (),
            }
        }
    }

    fn string(compiler: &mut Compiler, _scanner: &mut Scanner, _can_assign: bool) {
        if let Some(token) = &compiler.previous {
            let source = compiler
                .source
                .get((token.start + 1)..(token.start + token.length - 1));
            match source {
                Some(string) => {
                    let value = if let Some(existing_string) = compiler.strings.get(string) {
                        existing_string.to_string()
                    } else {
                        let value = String::from(string);
                        compiler.strings.insert(value.clone());
                        value
                    };
                    let index = compiler.add_constant(Value::Object(Object::String(value)));
                    compiler.emit_opcode(OpCode::Constant(index));
                }
                None => (),
            }
        }
    }

    fn grouping(compiler: &mut Compiler, scanner: &mut Scanner, _can_assgin: bool) {
        compiler.expression(scanner);
        compiler.consume(
            scanner,
            TokenKind::RightParen,
            "Expect a ')' after expression.",
        );
    }

    fn unary(compiler: &mut Compiler, scanner: &mut Scanner, _can_assign: bool) {
        let operator = compiler.previous.as_ref().unwrap().kind.clone();

        compiler.parse_precedence(Precedence::Unary, scanner);

        match operator {
            TokenKind::Minus => {
                compiler.emit_opcode(OpCode::Negate);
            }
            TokenKind::Bang => {
                compiler.emit_opcode(OpCode::Not);
            }
            _ => (),
        }
    }

    fn binary(compiler: &mut Compiler, scanner: &mut Scanner, _can_assign: bool) {
        let operator = compiler.previous.as_ref().unwrap().kind.clone();
        let rule = compiler.get_rule(&operator);
        compiler.parse_precedence(rule.precedence, scanner);

        match operator {
            TokenKind::Plus => compiler.emit_opcode(OpCode::Add),
            TokenKind::Minus => compiler.emit_opcode(OpCode::Subtract),
            TokenKind::Star => compiler.emit_opcode(OpCode::Multiply),
            TokenKind::Slash => compiler.emit_opcode(OpCode::Divide),
            TokenKind::BangEqual => {
                compiler.emit_opcode(OpCode::Equal);
                compiler.emit_opcode(OpCode::Not);
            }
            TokenKind::EqualEqual => compiler.emit_opcode(OpCode::Equal),
            TokenKind::Greater => compiler.emit_opcode(OpCode::Greater),
            TokenKind::GreaterEqual => {
                compiler.emit_opcode(OpCode::Less);
                compiler.emit_opcode(OpCode::Not);
            }
            TokenKind::Less => compiler.emit_opcode(OpCode::Less),
            TokenKind::LessEqual => {
                compiler.emit_opcode(OpCode::Greater);
                compiler.emit_opcode(OpCode::Not);
            }
            _ => (),
        }
    }

    fn call(compiler: &mut Compiler, scanner: &mut Scanner, _can_assign: bool) {
        let arg_count = compiler.argument_list(scanner);
        compiler.emit_opcode(OpCode::Call(arg_count));
    }

    fn argument_list(&mut self, scanner: &mut Scanner) -> usize {
        let mut arg_count = 0;

        if !self.matches(TokenKind::RightParen, scanner) {
            loop {
                self.expression(scanner);
                arg_count += 1;
                if !self.matches(TokenKind::Comma, scanner) {
                    break;
                }
            }
        }
        self.consume(
            scanner,
            TokenKind::RightParen,
            "Expect ')' after arguments.",
        );
        arg_count
    }

    fn and(compiler: &mut Compiler, scanner: &mut Scanner, _can_assign: bool) {
        let end_jump = compiler.emit_jump(OpCode::JumpIfFalse(0));

        compiler.emit_opcode(OpCode::Pop);

        compiler.parse_precedence(Precedence::And, scanner);

        compiler.patch_jump(end_jump);
    }

    fn or(compiler: &mut Compiler, scanner: &mut Scanner, _can_assign: bool) {
        let else_jump = compiler.emit_jump(OpCode::JumpIfFalse(0));
        let end_jump = compiler.emit_jump(OpCode::Jump(0));

        compiler.patch_jump(else_jump);
        compiler.emit_opcode(OpCode::Pop);

        compiler.parse_precedence(Precedence::Or, scanner);
        compiler.patch_jump(end_jump);
    }

    fn literal(compiler: &mut Compiler, _scanner: &mut Scanner, _can_assign: bool) {
        let operator = compiler.previous.as_ref().unwrap().kind.clone();
        match operator {
            TokenKind::Nil => compiler.emit_opcode(OpCode::Nil),
            TokenKind::True => compiler.emit_opcode(OpCode::True),
            TokenKind::False => compiler.emit_opcode(OpCode::False),
            _ => (),
        }
    }

    fn variable(compiler: &mut Compiler, scanner: &mut Scanner, can_assign: bool) {
        compiler.named_variable(scanner, can_assign);
    }

    fn named_variable(&mut self, scanner: &mut Scanner, can_assign: bool) {
        let token = self.previous.clone().unwrap();
        let get_op;
        let set_op;

        if let Some(index) = self.resolve_local(self.current_state_index(), &token) {
            get_op = OpCode::GetLocal(index);
            set_op = OpCode::SetLocal(index);
        } else if let Some(index) = self.resolve_upvalue(self.current_state_index(), &token) {
            get_op = OpCode::GetUpvalue(index);
            set_op = OpCode::SetUpvalue(index);
        } else {
            let index = self.identifier_constant(&token);
            get_op = OpCode::GetGlobal(index);
            set_op = OpCode::SetGlobal(index);
        }

        if can_assign && self.matches(TokenKind::Equal, scanner) {
            self.expression(scanner);
            self.emit_opcode(set_op);
        } else {
            self.emit_opcode(get_op);
        }
    }

    fn resolve_local(&self, state_idx: usize, name: &Token) -> Option<usize> {
        let state = self.states.get(state_idx);
        match state {
            Some(state) => {
                for (i, local) in state.locals.iter().enumerate().rev() {
                    if self.identifiers_equal(&local.name, name) {
                        return Some(i);
                    }
                }
                None
            }
            None => None
        }
    }

    fn resolve_upvalue(&mut self, state_idx: usize, name: &Token) -> Option<usize> {
        if let Some(state) = self.states.get(state_idx) {
            if let Some(enclosing_idx) = state.enclosing {
                if let Some(index) = self.resolve_local(enclosing_idx, name) {
                    println!("Adding local upvalue for {:?}", &self.source[name.as_range()]);
                    return self.add_upvalue(index, true);
                }

                if let Some(index) = self.resolve_upvalue(enclosing_idx, name) {
                    println!("Adding non-local upvalue for {:?}", &self.source[name.as_range()]);
                    return self.add_upvalue(index, false);
                }
            }
        }

        None
    }

    fn add_upvalue(&mut self, index: usize, local: bool) -> Option<usize> {
        let mut upvalue_count = self.state().function.upvalue_count;
        for i in 0..upvalue_count {
            let upvalue = &self.upvalues[i];
            if upvalue.local == local && upvalue.index == index {
                return Some(i);
            }
        }
        self.upvalues.push(Upvalue::new(local, index));
        upvalue_count += 1;
        self.state_mut().function.upvalue_count = upvalue_count;
        Some(upvalue_count)
    }

    fn identifiers_equal(&self, lhs: &Token, rhs: &Token) -> bool {
        if lhs.length != rhs.length {
            return false;
        }

        if self.source[lhs.as_range()] == self.source[rhs.as_range()] {
            return true;
        }
        false
    }
}
