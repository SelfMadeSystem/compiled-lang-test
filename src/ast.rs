use crate::tokens::Identifier;
use anyhow::{anyhow, Error, Result};

#[derive(Debug, PartialEq, Clone)]
pub struct Ast {
    pub kind: AstKind,
    pub line: usize,
    pub column: usize,
}

impl Ast {
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
        if let AstKind::Int(value) = &self.kind {
            Ok(*value)
        } else {
            self.err("Expected integer")
        }
    }

    pub fn as_float(&self) -> Result<f64> {
        if let AstKind::Float(value) = &self.kind {
            Ok(*value)
        } else {
            self.err("Expected float")
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        if let AstKind::Bool(value) = &self.kind {
            Ok(*value)
        } else {
            self.err("Expected boolean")
        }
    }

    pub fn as_char(&self) -> Result<char> {
        if let AstKind::Char(value) = &self.kind {
            Ok(*value)
        } else {
            self.err("Expected char")
        }
    }

    pub fn as_string(&self) -> Result<String> {
        if let AstKind::String(value) = &self.kind {
            Ok(value.clone())
        } else {
            self.err("Expected string")
        }
    }

    pub fn as_array(&self) -> Result<Vec<Ast>> {
        if let AstKind::Array(value) = &self.kind {
            Ok(value.clone())
        } else {
            self.err("Expected array")
        }
    }

    pub fn as_identifier(&self) -> Result<Identifier> {
        if let AstKind::Identifier(identifier) = &self.kind {
            Ok(identifier.clone())
        } else {
            self.err("Expected identifier")
        }
    }

    pub fn as_call(&self) -> Result<(Identifier, Vec<Ast>)> {
        if let AstKind::Call { name, args } = &self.kind {
            Ok((name.clone(), args.clone()))
        } else {
            self.err("Expected call")
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Array(Vec<Ast>),
    Identifier(Identifier),
    Call { name: Identifier, args: Vec<Ast> },
}
