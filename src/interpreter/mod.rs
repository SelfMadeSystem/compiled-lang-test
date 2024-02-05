use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::Result;

use crate::{
    parser::ast::{ParsedAst, ParsedAstKind},
    tokens::IdentifierKind,
};

use self::{
    ast::{ItpAst, ItpAstKind},
    macros::Macro,
    scope::Scope,
    value::{IFPCheck, ItpConstantValue, ItpFunctionValue, ItpValue, UnItpedFunctionValue},
};

pub mod ast;
pub mod macros;
pub mod scope;
pub mod value;

#[derive(Debug)]
pub struct Interpreter {
    pub scope: Rc<RefCell<Scope>>,
    pub macros: HashMap<String, Macro>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            scope: Rc::new(RefCell::new(Scope::new())),
            macros: macros::macros(),
        }
    }

    fn interpret_ast(
        &mut self,
        ast: &ParsedAst,
        scope: &Rc<RefCell<Scope>>,
    ) -> Result<Vec<ItpAst>> {
        let line = ast.line;
        let column = ast.column;
        match &ast.kind {
            ParsedAstKind::Int(value) => Ok(vec![ItpAst {
                kind: ItpAstKind::Constant(ItpConstantValue::Int(*value)),
                line,
                column,
            }]),
            ParsedAstKind::Float(value) => Ok(vec![ItpAst {
                kind: ItpAstKind::Constant(ItpConstantValue::Float(*value)),
                line,
                column,
            }]),
            ParsedAstKind::String(value) => Ok(vec![ItpAst {
                kind: ItpAstKind::Constant(ItpConstantValue::String(value.clone())),
                line,
                column,
            }]),
            ParsedAstKind::Char(value) => Ok(vec![ItpAst {
                kind: ItpAstKind::Constant(ItpConstantValue::Char(*value)),
                line,
                column,
            }]),
            ParsedAstKind::Bool(value) => Ok(vec![ItpAst {
                kind: ItpAstKind::Constant(ItpConstantValue::Bool(*value)),
                line,
                column,
            }]),
            ParsedAstKind::Array(values) => {
                let mut result = vec![];
                for value in values {
                    result.extend(self.interpret_ast(value, scope)?);
                }
                Ok(result)
            }
            ParsedAstKind::Identifier(identifier) => match identifier.kind {
                IdentifierKind::Variable => {
                    let scope = scope.borrow();
                    let value = scope.get(&identifier.name).ok_or_else(|| {
                        ast.error(&format!("Variable {} not found", identifier.name))
                    })?;
                    if let ItpValue::Constant(c) = value.as_ref() {
                        Ok(vec![ItpAst {
                            kind: ItpAstKind::Constant(c.clone()),
                            line,
                            column,
                        }])
                    } else {
                        Ok(vec![ItpAst {
                            kind: ItpAstKind::Variable {
                                name: identifier.clone(),
                                result: value.get_type(),
                            },
                            line,
                            column,
                        }])
                    }
                }
                IdentifierKind::Macro => ast.err("Macro not allowed here"),
                IdentifierKind::Type => ast.err("Type not allowed here"),
            },
            ParsedAstKind::Call { name, args } => match name.kind {
                IdentifierKind::Macro => {
                    let macro_ = self
                        .macros
                        .get(&name.name)
                        .ok_or_else(|| ast.error(&format!("Macro {} not found", name.name)))?;

                    macro_(args, self)
                }
                IdentifierKind::Variable => {
                    let func = scope
                        .borrow()
                        .get(&name.name)
                        .ok_or_else(|| ast.error(&format!("Function {} not found", name.name)))?;

                    match func.as_ref() {
                        ItpValue::Function(ItpFunctionValue {
                            parameters,
                            return_type,
                            ..
                        }) | ItpValue::UnItpedFunction(UnItpedFunctionValue {
                            parameters,
                            return_type,
                            ..
                        }) => {
                            let new_scope = Scope::new_child(scope.clone());
                            let new_scope = Rc::new(RefCell::new(new_scope));
                            let mut result = vec![];
                            for arg in args {
                                result.extend(self.interpret_ast(arg, &new_scope)?);
                            }

                            match parameters
                                .check_params(&result.iter().map(|a| a.get_type()).collect())
                            {
                                IFPCheck::Ok => Ok(()),
                                IFPCheck::NotEnoughParameters => ast.err("Not enough parameters"),
                                IFPCheck::TooManyParameters => ast.err("Too many parameters"),
                                IFPCheck::WrongType(i, got, expected) => ast.err(&format!(
                                    "Wrong type for parameter {}: got {:?}, expected {:?}",
                                    i, got, expected
                                )),
                            }?;

                            Ok(vec![ItpAst {
                                kind: ItpAstKind::Call {
                                    function: name.clone(),
                                    arguments: result,
                                    result: return_type.clone(),
                                },
                                line,
                                column,
                            }])
                        }
                        _ => ast.err(&format!("{} is not a function", name.name)),
                    }
                }
                IdentifierKind::Type => ast.err("Type not allowed here"),
            },
        }
    }

    fn interpret_uninterpreted_functions(&mut self) -> Result<()> {
        let mut new_functions = HashMap::new();
        for (name, value) in self.scope.clone().borrow().bindings.iter() {
            if let ItpValue::UnItpedFunction(UnItpedFunctionValue {
                name: fn_name,
                parameters,
                body,
                return_type,
            }) = value.as_ref()
            {
                let new_scope = Scope::new_child(self.scope.clone());
                let new_scope = Rc::new(RefCell::new(new_scope));

                for (name, ty) in parameters.parameters.iter() {
                    new_scope
                        .borrow_mut()
                        .set(name.clone(), Rc::new(ItpValue::Temp(ty.clone())))?;
                }

                let mut interpreted_body = vec![];

                for ast in body {
                    interpreted_body.extend(self.interpret_ast(ast, &new_scope)?);
                }

                new_functions.insert(
                    name.clone(),
                    ItpValue::Function(ItpFunctionValue {
                        name: fn_name.clone(),
                        parameters: parameters.clone(),
                        body: interpreted_body,
                        return_type: return_type.clone(),
                    }),
                );
            }
        }

        for (name, value) in new_functions {
            self.scope.borrow_mut().replace(name, Rc::new(value))?;
        }

        Ok(())
    }

    pub fn interpret(&mut self, ast: &Vec<ParsedAst>) -> Result<()> {
        for ast in ast {
            self.interpret_ast(ast, &self.scope.clone())?;
        }

        self.interpret_uninterpreted_functions()?;

        Ok(())
    }
}
