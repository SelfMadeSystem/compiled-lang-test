use crate::tokens::Identifier;

use super::statement::Statement;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Temp,
    Constant(ConstantValue),
    Named(Identifier),
    // Type(TypeValue),
    Function(FunctionValue),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Array(Vec<Value>),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionValue {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<Statement>,
}
