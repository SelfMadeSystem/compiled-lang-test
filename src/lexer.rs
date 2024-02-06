use crate::tokens::{Identifier, IdentifierKind, DELIMITERS};

use super::tokens::{Token, TokenKind};
use anyhow::{anyhow, Error, Result};

pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
    saved_line: usize,
    saved_column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
            saved_line: 1,
            saved_column: 1,
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

    fn save_position(&mut self) {
        self.saved_line = self.line;
        self.saved_column = self.column;
    }

    fn error(&self, message: &str) -> Error {
        anyhow!(
            "Error at line {} column {}: {}",
            self.saved_line,
            self.saved_column,
            message
        )
    }

    fn error_here(&mut self, message: &str) -> Error {
        self.save_position();
        self.error(message)
    }

    fn err<T>(&self, message: &str) -> Result<T> {
        Err(self.error(message))
    }

    fn err_here<T>(&mut self, message: &str) -> Result<T> {
        Err(self.error_here(message))
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

    fn read_number(&mut self) -> Result<Token> {
        let mut number = String::new();
        let mut float = false;
        self.save_position();
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() || ch == '.' {
                if ch == '.' {
                    if float {
                        return self.err_here("Unexpected '.' in number");
                    }
                    float = true;
                }
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        // TODO: Allow for more than just floats
        // if float {
        match number.parse::<f64>() {
            Ok(f) => Ok(Token {
                kind: TokenKind::Float(f),
                line: self.saved_line,
                column: self.saved_column,
            }),
            Err(_) => self.err("Invalid float"),
        }
        // } else {
        //     match number.parse::<i64>() {
        //         Ok(i) => Ok(Token {
        //             kind: TokenKind::Int(i),
        //             line: self.saved_line,
        //             column: self.saved_column,
        //         }),
        //         Err(_) => self.err("Invalid integer"),
        //     }
        // }
    }

    /// Reads just a single char that's part of a string or character literal.
    /// E.g., in `'a'`, this function would read the `a`.
    fn read_single_char(&mut self, skip_nl: bool) -> Result<Option<char>> {
        self.save_position();
        let mut ch = self
            .current_char()
            .ok_or_else(|| self.error("Unexpected EOF"))?;
        self.advance();
        match ch {
            '\\' => {
                ch = self
                    .current_char()
                    .ok_or_else(|| self.error("Unexpected EOF"))?;
                self.advance();
                ch = match ch {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    '0' => '\0',
                    '\n' if skip_nl => {
                        return Ok(None);
                    }
                    'u' => {
                        let mut hex = String::new();
                        for _ in 0..4 {
                            let ch = self
                                .current_char()
                                .ok_or_else(|| self.error("Unexpected EOF"))?;
                            if !ch.is_ascii_hexdigit() {
                                return self.err_here("Invalid unicode escape sequence");
                            }
                            hex.push(ch);
                            self.advance();
                        }
                        match u32::from_str_radix(&hex, 16) {
                            Ok(n) => match std::char::from_u32(n) {
                                Some(ch) => ch,
                                None => return self.err_here("Invalid unicode escape sequence"),
                            },
                            Err(_) => return self.err_here("Invalid unicode escape sequence"),
                        }
                    }
                    _ => return self.err_here("Invalid escape sequence"),
                };
            }
            '\n' if !skip_nl => return self.err_here("Unexpected newline"),
            _ => {}
        }
        Ok(Some(ch))
    }

    fn read_char_literal(&mut self) -> Result<Token> {
        self.save_position();
        if self.current_char() != Some('\'') {
            return self.err_here("Expected single quote");
        }
        self.advance();
        let ch = self.read_single_char(false)?;
        if self.current_char() != Some('\'') {
            return self.err_here("Expected single quote");
        }
        self.advance();
        Ok(Token {
            kind: TokenKind::Char(ch.unwrap()),
            line: self.saved_line,
            column: self.saved_column,
        })
    }

    fn read_string_literal(&mut self) -> Result<Token> {
        self.save_position();
        if self.current_char() != Some('"') {
            return self.err_here("Expected double quote");
        }
        self.advance();
        let mut string = String::new();
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::String(string),
                    line: self.saved_line,
                    column: self.saved_column,
                });
            }
            if let Some(ch) = self.read_single_char(true)? {
                string.push(ch);
            }
        }
        self.err_here("Unexpected EOF")
    }

    fn read_identifier(&mut self, first: char) -> Token {
        self.save_position();
        let mut identifier = String::new();
        let kind = match first {
            '@' => IdentifierKind::Macro,
            '$' => IdentifierKind::Type,
            _ => IdentifierKind::Variable,
        };

        if kind != IdentifierKind::Variable {
            self.advance();
        }

        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() || DELIMITERS.contains(ch) {
                break;
            }
            identifier.push(ch);
            self.advance();
        }

        if kind == IdentifierKind::Variable {
            match identifier.as_str() {
                "true" => {
                    return Token {
                        kind: TokenKind::Bool(true),
                        line: self.saved_line,
                        column: self.saved_column,
                    }
                }
                "false" => {
                    return Token {
                        kind: TokenKind::Bool(false),
                        line: self.saved_line,
                        column: self.saved_column,
                    }
                }
                _ => {}
            }
        }

        Token {
            kind: TokenKind::Identifier(Identifier {
                name: identifier,
                kind,
            }),
            line: self.saved_line,
            column: self.saved_column,
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        let token = match self.current_char() {
            Some(ch) => match ch {
                c if DELIMITERS.contains(c) => {
                    self.advance();
                    Token {
                        kind: TokenKind::Delimiter(ch),
                        line: self.line,
                        column: self.column,
                    }
                }
                '"' => self.read_string_literal()?,
                '\'' => self.read_char_literal()?,
                '0'..='9' => self.read_number()?,
                c => self.read_identifier(c),
            },
            None => Token {
                kind: TokenKind::EOF,
                line: self.line,
                column: self.column,
            },
        };
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
