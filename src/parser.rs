use crate::ast::BinaryOp;

use super::ast::{Ast, AstKind};
use super::tokens::{Token, TokenKind};
use anyhow::{anyhow, Result};

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

    fn parse_number(&mut self) -> Result<Ast> {
        let token = self
            .current_token()
            .ok_or_else(|| anyhow!("Unexpected EOF"))?;
        let line = token.line;
        let column = token.column;
        let number = token.kind.number().ok_or_else(|| {
            anyhow!(
                "Expected number at line {}, column {}. Found: {}",
                line,
                column,
                token.kind_string()
            )
        })?;

        self.advance();
        Ok(Ast {
            kind: AstKind::Number(number),
            line,
            column,
        })
    }

    fn parse_primary(&mut self) -> Result<Ast> {
        match self
            .current_token()
            .ok_or_else(|| anyhow!("Unexpected EOF"))?
            .kind
        {
            TokenKind::Number(_) => self.parse_number(),
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr(0)?;
                if self
                    .current_token()
                    .ok_or_else(|| anyhow!("Unexpected EOF"))?
                    .kind
                    != TokenKind::RParen
                {
                    return Err(anyhow!(
                        "Expected ')' at line {}, column {}",
                        expr.line,
                        expr.column
                    ));
                }
                self.advance();
                Ok(expr)
            }
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_primary()?;
                match expr.kind {
                    AstKind::Number(n) => Ok(Ast {
                        kind: AstKind::Number(-n),
                        line: expr.line,
                        column: expr.column,
                    }),
                    AstKind::BinaryOp { op, lhs, rhs } => {
                        Ok(Ast {
                            kind: AstKind::BinaryOp {
                                op: BinaryOp::Sub,
                                lhs: Box::new(Ast {
                                    kind: AstKind::Number(0.0),
                                    line: expr.line,
                                    column: expr.column,
                                }),
                                rhs: Box::new(Ast {
                                    kind: AstKind::BinaryOp { op, lhs, rhs },
                                    line: expr.line,
                                    column: expr.column,
                                }),
                            },
                            line: expr.line,
                            column: expr.column,
                        })
                    }
                }
            }
            _ => {
                let token = self
                    .current_token()
                    .ok_or_else(|| anyhow!("Unexpected EOF"))?;
                Err(anyhow!(
                    "Unexpected '(', Number, or '-' at line {}, column {}. Found: {}",
                    token.line,
                    token.column,
                    token.kind_string()
                ))
            }
        }
    }

    fn parse_expr(&mut self, precedence: usize) -> Result<Ast> {
        let mut lhs = self.parse_primary()?;

        while let Some(token) = self.current_token() {
            if token.kind == TokenKind::RParen {
                break;
            }

            let line = token.line;
            let column = token.column;
            let op = token.kind.binary_op().ok_or_else(|| {
                anyhow!(
                    "Expected binary operator at line {}, column {}. Found: {}",
                    line,
                    column,
                    token.kind_string()
                )
            })?;
            let token_precedence = op.precedence();

            if token_precedence < precedence {
                return Ok(lhs);
            }

            self.advance();
            let rhs = self.parse_expr(token_precedence + 1)?;
            lhs = Ast {
                kind: AstKind::BinaryOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                line,
                column,
            };
        }

        Ok(lhs)
    }

    pub fn parse(&mut self) -> Result<Ast> {
        self.parse_expr(0)
    }
}
