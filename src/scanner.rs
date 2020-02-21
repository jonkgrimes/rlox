use crate::token::{Token, TokenKind};

pub struct Scanner<'a> {
    source: &'a str,
    pub start: usize,
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
        self.skip_whitespace();

        self.start = self.current;

        if self.at_end() {
            return self.make_token(TokenKind::Eof);
        }

        let c = self.advance();

        if c.map_or(false, is_alpha) {
            return self.identifier();
        }

        if c.map_or(false, is_digit) {
            return self.number();
        }

        match c {
            Some("(") => self.make_token(TokenKind::LeftParen),
            Some(")") => self.make_token(TokenKind::RightParen),
            Some("{") => self.make_token(TokenKind::LeftBrace),
            Some("}") => self.make_token(TokenKind::RightBrace),
            Some(";") => self.make_token(TokenKind::Semicolon),
            Some(",") => self.make_token(TokenKind::Comma),
            Some(".") => self.make_token(TokenKind::Dot),
            Some("-") => self.make_token(TokenKind::Minus),
            Some("+") => self.make_token(TokenKind::Plus),
            Some("/") => self.make_token(TokenKind::Slash),
            Some("*") => self.make_token(TokenKind::Star),
            Some("!") => {
                if self.matches("=") {
                    self.make_token(TokenKind::BangEqual)
                } else {
                    self.make_token(TokenKind::Bang)
                }
            }
            Some("=") => {
                if self.matches("=") {
                    self.make_token(TokenKind::EqualEqual)
                } else {
                    self.make_token(TokenKind::Equal)
                }
            }
            Some("<") => {
                if self.matches("=") {
                    self.make_token(TokenKind::LessEqual)
                } else {
                    self.make_token(TokenKind::Less)
                }
            }
            Some(">") => {
                if self.matches("=") {
                    self.make_token(TokenKind::GreaterEqual)
                } else {
                    self.make_token(TokenKind::Greater)
                }
            }
            Some("&") => {
                if self.matches("&") {
                    self.make_token(TokenKind::And)
                } else {
                    self.make_token(TokenKind::And)
                }
            }
            Some("\"") => self.string(),
            _ => Token::error("Unexpected token", self.start, self.current, self.line),
        }
    }

    pub fn at_end(&mut self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> Option<&str> {
        self.current += 1;
        self.source.get((self.current - 1)..self.current)
    }

    fn peek(&mut self) -> Option<&str> {
        let next = self.current;
        self.source.get(next..next + 1)
    }

    fn peek_next(&mut self) -> Option<&str> {
        let next = self.current + 1;
        self.source.get(next..next + 1)
    }

    fn matches(&mut self, expected: &str) -> bool {
        if self.at_end() {
            return false;
        }

        let pos = self.current;
        if let Some(next) = self.source.get(pos..pos + 1) {
            if next != expected {
                return false;
            }
        }

        self.current += 1;
        true
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            start: self.start,
            length: self.current - self.start,
            line: self.line,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(" ") => {
                    self.advance();
                }
                Some("\r") => {
                    self.advance();
                }
                Some("\t") => {
                    self.advance();
                }
                Some("\n") => {
                    self.advance();
                    self.line += 1;
                }
                Some("/") => {
                    if self.peek_next() == Some("/") {
                        while self.peek() != Some("\n") && !self.at_end() {
                            self.advance();
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != Some("\"") && !self.at_end() {
            if self.peek() == Some("\n") {
                self.line += 1;
            }
            self.advance();
        }

        if self.at_end() {
            return Token::error("Unterminated string.", self.start, self.current, self.line);
        }

        self.advance(); // closing "

        self.make_token(TokenKind::String)
    }

    fn number(&mut self) -> Token {
        while self.peek().map_or(false, is_digit) {
            self.advance();
        }

        if self.peek() == Some(".") && self.peek_next().map_or(false, is_digit) {
            self.advance();

            while self.peek().map_or(false, is_digit) {
                self.advance();
            }
        }

        self.make_token(TokenKind::Number)
    }

    fn identifier(&mut self) -> Token {
        while self.peek().map_or(false, is_alpha) || self.peek().map_or(false, is_digit) {
            self.advance();
        }

        let identifier = self.identifier_type();
        return self.make_token(identifier);
    }

    fn identifier_type(&mut self) -> TokenKind {
        let c = self.source.get(self.start..self.start + 1);
        match c {
            Some("a") => self.check_keyword(2, 2, "nd", TokenKind::And),
            Some("c") => self.check_keyword(2, 4, "class", TokenKind::Class),
            Some("e") => self.check_keyword(3, 3, "lse", TokenKind::Else),
            Some("f") => {
                if self.current - self.start > 1 {
                    let c = self.source.get((self.start + 1)..(self.start + 2));
                    match c {
                        Some("a") => self.check_keyword(3, 3, "lse", TokenKind::False),
                        Some("o") => self.check_keyword(1, 1, "r", TokenKind::For),
                        Some("u") => self.check_keyword(1, 1, "n", TokenKind::Fun),
                        _ => TokenKind::Error("Unexpected character".to_string()),
                    }
                } else {
                    TokenKind::Error("Unexpected character".to_string())
                }
            }
            Some("i") => self.check_keyword(1, 1, "f", TokenKind::If),
            Some("n") => self.check_keyword(2, 2, "il", TokenKind::Nil),
            Some("o") => self.check_keyword(2, 1, "o", TokenKind::Or),
            Some("p") => self.check_keyword(4, 4, "rint", TokenKind::Print),
            Some("r") => self.check_keyword(5, 5, "eturn", TokenKind::Return),
            Some("s") => self.check_keyword(4, 4, "uper", TokenKind::Super),
            Some("t") => {
                if self.current - self.start > 1 {
                    let c = self.source.get((self.start + 1)..(self.start + 2));
                    match c {
                        Some("h") => self.check_keyword(2, 2, "is", TokenKind::This),
                        Some("r") => self.check_keyword(2, 2, "ue", TokenKind::True),
                        _ => TokenKind::Identifier,
                    }
                } else {
                    TokenKind::Error("Unexpected character".to_string())
                }
            }
            Some("v") => self.check_keyword(2, 2, "ar", TokenKind::Var),
            Some("w") => self.check_keyword(4, 4, "hile", TokenKind::While),
            _ => TokenKind::Identifier,
        }
    }

    fn check_keyword(
        &mut self,
        start: usize,
        length: usize,
        rest: &str,
        kind: TokenKind,
    ) -> TokenKind {
        let start = self.current - start;
        let range = start..(start + length);
        let source = self.source.get(range);
        if source == Some(rest) {
            return kind;
        }
        TokenKind::Identifier
    }

    pub fn line(&self) -> i32 {
        self.line
    }
}

fn is_digit(c: &str) -> bool {
    c >= "0" && c <= "9"
}

fn is_alpha(c: &str) -> bool {
    (c >= "a" && c <= "z") || (c >= "A" && c <= "Z") || (c == "_")
}
