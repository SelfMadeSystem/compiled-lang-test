use super::tokens::{Token, TokenKind};
use anyhow::{anyhow, Result};

pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> String {
        let mut number = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() || ch == '.' {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        number
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        let token = match self.current_char() {
            Some('i') => Token {
                kind: TokenKind::Input,
                line: self.line,
                column: self.column,
            },
            Some('+') => Token {
                kind: TokenKind::Plus,
                line: self.line,
                column: self.column,
            },
            Some('-') => Token {
                kind: TokenKind::Minus,
                line: self.line,
                column: self.column,
            },
            Some('*') => Token {
                kind: TokenKind::Star,
                line: self.line,
                column: self.column,
            },
            Some('/') => Token {
                kind: TokenKind::Slash,
                line: self.line,
                column: self.column,
            },
            Some('(') => Token {
                kind: TokenKind::LParen,
                line: self.line,
                column: self.column,
            },
            Some(')') => Token {
                kind: TokenKind::RParen,
                line: self.line,
                column: self.column,
            },
            Some(ch) if ch.is_ascii_digit() => Token {
                kind: TokenKind::Number(self.read_number()),
                line: self.line,
                column: self.column,
            },
            Some(ch) => {
                return Err(anyhow!(
                    "unexpected character '{}' at line {} column {}",
                    ch, self.line, self.column
                ))
            }
            None => Token {
                kind: TokenKind::EOF,
                line: self.line,
                column: self.column,
            },
        };
        self.advance();
        Ok(token)
    }

    pub fn lex(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token.kind == TokenKind::EOF {
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }
}
