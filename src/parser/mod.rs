use super::tokens::{Token, TokenKind};
use anyhow::{anyhow, Result};
use ast::{ParsedAst, ParsedAstKind};
pub mod ast;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn parse_literal(&mut self) -> Result<ParsedAst> {
        let token = self
            .current_token()
            .ok_or_else(|| anyhow!("Unexpected EOF"))?;
        let line = token.line;
        let column = token.column;

        match &token.kind {
            TokenKind::Int(n) => {
                let n = *n;
                self.advance();
                Ok(ParsedAst {
                    kind: ParsedAstKind::Int(n),
                    line,
                    column,
                })
            }
            TokenKind::Float(n) => {
                let n = *n;
                self.advance();
                Ok(ParsedAst {
                    kind: ParsedAstKind::Float(n),
                    line,
                    column,
                })
            }
            TokenKind::Bool(b) => {
                let b = *b;
                self.advance();
                Ok(ParsedAst {
                    kind: ParsedAstKind::Bool(b),
                    line,
                    column,
                })
            }
            TokenKind::Char(c) => {
                let c = *c;
                self.advance();
                Ok(ParsedAst {
                    kind: ParsedAstKind::Char(c),
                    line,
                    column,
                })
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(ParsedAst {
                    kind: ParsedAstKind::String(s),
                    line,
                    column,
                })
            }
            TokenKind::Identifier(id) => {
                let id = id.clone();
                self.advance();
                Ok(ParsedAst {
                    kind: ParsedAstKind::Identifier(id),
                    line,
                    column,
                })
            }
            _ => Err(anyhow!(
                "Expected literal at line {}, column {}. Found: {}",
                line,
                column,
                token
            )),
        }
    }

    fn parse_array(&mut self) -> Result<ParsedAst> {
        let token = self
            .current_token()
            .ok_or_else(|| anyhow!("Unexpected EOF"))?;
        let line = token.line;
        let column = token.column;

        if token.kind != TokenKind::Delimiter('[') {
            return Err(anyhow!(
                "Expected '[' at line {}, column {}. Found: {}",
                line,
                column,
                token
            ));
        }

        self.advance();

        let mut elements = Vec::new();

        loop {
            let token = self
                .current_token()
                .ok_or_else(|| anyhow!("Unexpected EOF"))?;
            let line = token.line;
            let column = token.column;

            if token.kind == TokenKind::Delimiter(']') {
                self.advance();
                break;
            }

            let element = self.parse_expression()?;
            elements.push(element);

            let token = self
                .current_token()
                .ok_or_else(|| anyhow!("Unexpected EOF"))?;
            if token.kind == TokenKind::Delimiter(']') {
                self.advance();
                break;
            } else if token.kind != TokenKind::Delimiter(',') {
                return Err(anyhow!(
                    "Expected ',' or ']' at line {}, column {}. Found: {}",
                    line,
                    column,
                    token
                ));
            }

            self.advance();
        }

        Ok(ParsedAst {
            kind: ParsedAstKind::Array(elements),
            line,
            column,
        })
    }

    fn parse_call(&mut self) -> Result<ParsedAst> {
        let token = self
            .current_token()
            .ok_or_else(|| anyhow!("Unexpected EOF"))?;
        let line = token.line;
        let column = token.column;

        if token.kind != TokenKind::Delimiter('(') {
            return Err(anyhow!(
                "Expected '(' at line {}, column {}. Found: {}",
                line,
                column,
                token
            ));
        }

        self.advance();

        let name = match self.current_token() {
            Some(Token {
                kind: TokenKind::Identifier(name),
                ..
            }) => name.clone(),
            Some(token) => {
                return Err(anyhow!(
                    "Expected identifier at line {}, column {}. Found: {}",
                    token.line,
                    token.column,
                    token
                ))
            }
            None => return Err(anyhow!("Unexpected EOF")),
        };

        self.advance();

        let mut args = Vec::new();

        loop {
            let token = self
                .current_token()
                .ok_or_else(|| anyhow!("Unexpected EOF"))?;

            if token.kind == TokenKind::Delimiter(')') {
                self.advance();
                break;
            }

            let arg = self.parse_expression()?;
            args.push(arg);

            let token = self
                .current_token()
                .ok_or_else(|| anyhow!("Unexpected EOF"))?;
            if token.kind == TokenKind::Delimiter(',') {
                self.advance();
            }
        }

        Ok(ParsedAst {
            kind: ParsedAstKind::Call { name, args },
            line,
            column,
        })
    }

    fn parse_expression(&mut self) -> Result<ParsedAst> {
        let token = self
            .current_token()
            .ok_or_else(|| anyhow!("Unexpected EOF"))?;
        let line = token.line;
        let column = token.column;

        match &token.kind {
            TokenKind::Delimiter('[') => self.parse_array(),
            TokenKind::Delimiter('(') => self.parse_call(),
            t if t.is_literal() => self.parse_literal(),
            _ => Err(anyhow!(
                "Expected expression at line {}, column {}. Found: {}",
                line,
                column,
                token
            )),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ParsedAst>> {
        let mut ast = Vec::new();

        while let Some(e) = self.current_token() {
            if e.kind == TokenKind::EOF {
                break;
            }
            let expr = self.parse_expression()?;
            ast.push(expr);
        }

        Ok(ast)
    }
}
