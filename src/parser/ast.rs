use crate::tokens::Identifier;
use anyhow::{anyhow, Error, Result};

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedAst {
    pub kind: ParsedAstKind,
    pub line: usize,
    pub column: usize,
}

impl ParsedAst {
    pub fn error(&self, message: &str) -> Error {
        anyhow!(
            "Error at line {} column {}: {}",
            self.line,
            self.column,
            message
        )
    }

    pub fn err<T>(&self, message: &str) -> Result<T> {
        Err(self.error(message))
    }

    pub fn as_int(&self) -> Result<i64> {
        if let ParsedAstKind::Int(value) = &self.kind {
            Ok(*value)
        } else {
            self.err("Expected integer")
        }
    }

    pub fn as_float(&self) -> Result<f64> {
        if let ParsedAstKind::Float(value) = &self.kind {
            Ok(*value)
        } else {
            self.err("Expected float")
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        if let ParsedAstKind::Bool(value) = &self.kind {
            Ok(*value)
        } else {
            self.err("Expected boolean")
        }
    }

    pub fn as_char(&self) -> Result<char> {
        if let ParsedAstKind::Char(value) = &self.kind {
            Ok(*value)
        } else {
            self.err("Expected char")
        }
    }

    pub fn as_string(&self) -> Result<String> {
        if let ParsedAstKind::String(value) = &self.kind {
            Ok(value.clone())
        } else {
            self.err("Expected string")
        }
    }

    pub fn as_array(&self) -> Result<Vec<ParsedAst>> {
        if let ParsedAstKind::Array(value) = &self.kind {
            Ok(value.clone())
        } else {
            self.err("Expected array")
        }
    }

    pub fn as_identifier(&self) -> Result<Identifier> {
        if let ParsedAstKind::Identifier(identifier) = &self.kind {
            Ok(identifier.clone())
        } else {
            self.err("Expected identifier")
        }
    }

    pub fn as_call(&self) -> Result<(Identifier, Vec<ParsedAst>)> {
        if let ParsedAstKind::Call { name, args } = &self.kind {
            Ok((name.clone(), args.clone()))
        } else {
            self.err("Expected call")
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedAstKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Array(Vec<ParsedAst>),
    Identifier(Identifier),
    Call {
        name: Identifier,
        args: Vec<ParsedAst>,
    },
}
