use std::collections::{HashMap, HashSet};

use crate::{parser::ast::ParsedAst, tokens::Identifier};

use super::ast::ItpAst;

#[derive(Debug, Clone, PartialEq)]
pub enum ItpValue {
    Param(u32, ItpTypeValue),
    Constant(ItpConstantValue),
    Named(Identifier, ItpTypeValue),
    // Type(TypeValue),
    Function(ItpFunctionValue),
    UnItpedFunction(UnItpedFunctionValue),
    NativeFunction(NativeFunctionValue),
}

impl ItpValue {
    pub fn get_type(&self) -> ItpTypeValue {
        match self {
            ItpValue::Param(_, t) => t.clone(),
            ItpValue::Constant(c) => c.get_type(),
            ItpValue::Named(_, t) => t.clone(),
            ItpValue::Function(f) => ItpTypeValue::Function {
                parameters: f.parameters.clone(),
                return_type: Box::new(f.return_type.clone()),
            },
            ItpValue::UnItpedFunction(f) => ItpTypeValue::Function {
                parameters: f.parameters.clone(),
                return_type: Box::new(f.return_type.clone()),
            },
            ItpValue::NativeFunction(f) => ItpTypeValue::Function {
                parameters: f.parameters.clone(),
                return_type: Box::new(f.return_type.clone()),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItpConstantValue {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    Array(Vec<ItpValue>),
}

impl ItpConstantValue {
    pub fn get_type(&self) -> ItpTypeValue {
        match self {
            ItpConstantValue::Int(_) => ItpTypeValue::Int,
            ItpConstantValue::Float(_) => ItpTypeValue::Float,
            ItpConstantValue::String(_) => ItpTypeValue::String,
            ItpConstantValue::Char(_) => ItpTypeValue::Char,
            ItpConstantValue::Bool(_) => ItpTypeValue::Bool,
            ItpConstantValue::Array(values) => {
                let mut types = values.iter().map(|v| v.get_type()).collect::<HashSet<_>>();

                if types.len() == 1 {
                    ItpTypeValue::Array(Box::new(types.drain().next().unwrap()))
                } else {
                    panic!("Array with different types")
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItpTypeValue {
    Int,
    Float,
    String,
    Char,
    Bool,
    Array(Box<ItpTypeValue>), // TODO: Absolutely should include length here
    Function {
        parameters: ItpFunctionParameters,
        return_type: Box<ItpTypeValue>,
    },
    Generic(String),
    Void,
}

impl ItpTypeValue {
    pub fn generic(name: &str) -> ItpTypeValue {
        ItpTypeValue::Generic(name.to_string())
    }

    pub fn replace_generics(&self, generics: &HashMap<String, ItpTypeValue>) -> ItpTypeValue {
        match self {
            ItpTypeValue::Generic(name) => match generics.get(name) {
                Some(t) => t.clone(),
                None => self.clone(),
            },
            ItpTypeValue::Array(t) => ItpTypeValue::Array(Box::new(t.replace_generics(generics))),
            ItpTypeValue::Function {
                parameters,
                return_type,
            } => ItpTypeValue::Function {
                parameters: parameters.clone(),
                return_type: Box::new(return_type.replace_generics(generics)),
            },
            _ => self.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItpFunctionParameters {
    pub generics: Vec<String>,
    pub parameters: Vec<(String, ItpTypeValue)>,
    pub variadic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IFPCheck {
    Ok(HashMap<String, ItpTypeValue>),
    NotEnoughParameters,
    TooManyParameters,
    WrongType(usize, ItpTypeValue, ItpTypeValue),
}

fn check_param(
    generics: &mut HashMap<String, ItpTypeValue>,
    i: usize,
    param: &ItpTypeValue,
    value: &ItpTypeValue,
) -> Result<(), IFPCheck> {
    match param {
        ItpTypeValue::Generic(name) => match generics.get(name) {
            None => {
                generics.insert(name.clone(), value.clone());
                Ok(())
            }
            Some(g) => {
                if g == value {
                    Ok(())
                } else {
                    Err(IFPCheck::WrongType(i, param.clone(), value.clone()))
                }
            }
        },
        ItpTypeValue::Array(t) => match value {
            ItpTypeValue::Array(v) => check_param(generics, i, t, v),
            _ => Err(IFPCheck::WrongType(i, param.clone(), value.clone())),
        },
        _ => {
            if param == value {
                Ok(())
            } else {
                Err(IFPCheck::WrongType(i, param.clone(), value.clone()))
            }
        }
    }
}

impl ItpFunctionParameters {
    pub fn check_params(&self, params: &Vec<ItpTypeValue>) -> IFPCheck {
        if params.len() < self.parameters.len() {
            return IFPCheck::NotEnoughParameters;
        }

        if !self.variadic && params.len() > self.parameters.len() {
            return IFPCheck::TooManyParameters;
        }

        let mut generics = HashMap::new();

        for (i, (_, t)) in self.parameters.iter().enumerate() {
            if let Err(r) = check_param(&mut generics, i, t, &params[i]) {
                return r;
            }
        }

        IFPCheck::Ok(generics)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnItpedFunctionValue {
    pub name: String,
    pub parameters: ItpFunctionParameters,
    pub body: Vec<ParsedAst>,
    pub return_type: ItpTypeValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItpFunctionValue {
    pub name: String,
    pub parameters: ItpFunctionParameters,
    pub body: Vec<ItpAst>,
    pub return_type: ItpTypeValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeFunctionValue {
    pub name: String,
    pub parameters: ItpFunctionParameters,
    pub return_type: ItpTypeValue,
    pub intrinsic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseFunctionValue {
    pub name: String,
    pub parameters: ItpFunctionParameters,
    pub return_type: ItpTypeValue,
}

pub trait ToBaseFunctionValue {
    fn to_base(&self) -> BaseFunctionValue;
}

impl ToBaseFunctionValue for ItpFunctionValue {
    fn to_base(&self) -> BaseFunctionValue {
        BaseFunctionValue {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            return_type: self.return_type.clone(),
        }
    }
}

impl ToBaseFunctionValue for UnItpedFunctionValue {
    fn to_base(&self) -> BaseFunctionValue {
        BaseFunctionValue {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            return_type: self.return_type.clone(),
        }
    }
}

impl ToBaseFunctionValue for NativeFunctionValue {
    fn to_base(&self) -> BaseFunctionValue {
        BaseFunctionValue {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            return_type: self.return_type.clone(),
        }
    }
}
