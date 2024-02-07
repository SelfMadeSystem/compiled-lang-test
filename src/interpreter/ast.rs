use anyhow::{anyhow, Error, Result};

use crate::tokens::Identifier;

use super::value::{ItpConstantValue, ItpTypeValue};

/// Like ParserAst, but with more information
#[derive(Debug, PartialEq, Clone)]
pub struct ItpAst {
    pub kind: ItpAstKind,
    pub line: usize,
    pub column: usize,
}

impl ItpAst {
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

    pub fn get_type(&self) -> ItpTypeValue {
        match &self.kind {
            ItpAstKind::Constant(value) => value.get_type(),
            ItpAstKind::Array(values) => {
                ItpTypeValue::Array(Box::new(if values.is_empty() {
                    ItpTypeValue::Void
                } else {
                    values[0].get_type()
                }))
            }
            ItpAstKind::Variable { result, .. } => result.clone(),
            ItpAstKind::SetVariable { .. } => ItpTypeValue::Void,
            ItpAstKind::Conditional { then, else_, .. } => {
                if then.get_type() != else_.get_type() {
                    ItpTypeValue::Void
                } else {
                    then.get_type()
                }
            }
            ItpAstKind::Loop { .. } => ItpTypeValue::Void,
            ItpAstKind::Param { result, .. } => result.clone(),
            ItpAstKind::Call { result, .. } => result.clone(),
        }
    }
}

/// The different kinds of AST nodes
#[derive(Debug, PartialEq, Clone)]
pub enum ItpAstKind {
    Constant(ItpConstantValue),
    Array(Vec<ItpAst>),
    Variable {
        name: Identifier,
        result: ItpTypeValue,
    },
    SetVariable {
        name: Identifier,
        value: Box<ItpAst>,
    },
    Param {
        position: u32,
        name: Identifier,
        result: ItpTypeValue,
    },
    Conditional {
        condition: Box<ItpAst>,
        then: Box<ItpAst>,
        else_: Box<ItpAst>,
    },
    Loop {
        condition: Box<ItpAst>,
        body: Box<ItpAst>,
    },
    Call {
        function: Identifier,
        arguments: Vec<ItpAst>,
        result: ItpTypeValue,
    },
}
