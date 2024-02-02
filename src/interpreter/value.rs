use crate::tokens::Identifier;

use super::statement::Statement;

#[derive(Debug, Clone, PartialEq)]
pub enum ItpValue {
    Temp,
    Constant(ItpConstantValue),
    Named(Identifier),
    // Type(TypeValue),
    Function(ItpFunctionValue),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItpConstantValue {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Array(Vec<ItpValue>),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItpTypeValue {
    Int,
    Float,
    String,
    Char,
    Bool,
    Array(Box<ItpTypeValue>),
    Function {
        parameters: ItpFunctionParameters,
        return_type: Box<ItpTypeValue>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItpFunctionParameters {
    pub parameters: Vec<(String, ItpTypeValue)>,
    pub variadic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItpFunctionValue {
    pub name: String,
    pub parameters: ItpFunctionParameters,
    pub body: Vec<Statement>,
    pub return_type: ItpTypeValue,
}
