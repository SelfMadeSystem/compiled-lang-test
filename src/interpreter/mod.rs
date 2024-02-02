use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::{anyhow, Result};

use crate::{
    ast::{Ast, AstKind},
    tokens::{Identifier, IdentifierKind},
};

use self::{
    macros::Macro,
    scope::Scope,
    statement::{Statement, StatementKind},
    value::{ConstantValue, Value},
};

mod macros;
mod scope;
mod statement;
mod value;

#[derive(Debug)]
pub struct Interpreter {
    scope: Rc<RefCell<Scope>>,
    macros: HashMap<String, Macro>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            scope: Rc::new(RefCell::new(Scope::new())),
            macros: macros::macros(),
        }
    }

    fn ast_to_statements(
        &mut self,
        ast: &Ast,
        scope: &Rc<RefCell<Scope>>,
    ) -> Result<Vec<Statement>> {
        match &ast.kind {
            AstKind::Call { name, args } => {
                // If the call is a macro, execute the macro
                match name.kind {
                    IdentifierKind::Macro => {
                        if let Some(macro_func) = self.macros.get(&name.name) {
                            return macro_func(&args, self);
                        } else {
                            return ast.err(&format!("Macro {} not found", name.name));
                        }
                    }
                    IdentifierKind::Type => return ast.err("Type not callable"),
                    _ => {}
                }

                // When one of the arguments is a call statement, we need to
                // assign the result to a temporary variable and use that
                // variable as an argument

                let mut statements = vec![];
                let mut arguments = vec![];

                for arg in args {
                    let statement = self.ast_to_statements(arg, scope)?;
                    statements.extend(statement);

                    let last = statements.pop().expect("No statement");

                    match last.kind {
                        StatementKind::Value { value } => {
                            arguments.push(value);
                        }
                        StatementKind::CallAssign { ref assign, .. } => {
                            arguments.push(Value::Named(assign.clone()));
                            statements.push(last);
                        }
                        _ => return ast.err("Invalid argument"),
                    }
                }

                statements.push(Statement {
                    kind: StatementKind::CallAssign {
                        name: name.clone(),
                        arguments,
                        assign: Identifier::new_variable(
                            &scope.borrow_mut().new_temp_name(),
                        ),
                    },
                });

                Ok(statements)
            }
            AstKind::Int(i) => Ok(vec![Statement {
                kind: StatementKind::Value {
                    value: Value::Constant(ConstantValue::Int(*i)),
                },
            }]),
            AstKind::Float(f) => Ok(vec![Statement {
                kind: StatementKind::Value {
                    value: Value::Constant(ConstantValue::Float(*f)),
                },
            }]),
            AstKind::Bool(b) => Ok(vec![Statement {
                kind: StatementKind::Value {
                    value: Value::Constant(ConstantValue::Bool(*b)),
                },
            }]),
            AstKind::Char(c) => Ok(vec![Statement {
                kind: StatementKind::Value {
                    value: Value::Constant(ConstantValue::Char(*c)),
                },
            }]),
            AstKind::String(s) => Ok(vec![Statement {
                kind: StatementKind::Value {
                    value: Value::Constant(ConstantValue::String(s.clone())),
                },
            }]),
            AstKind::Identifier(id) => {
                match id.kind {
                    IdentifierKind::Macro => {
                        return ast.err("Macro is not a value");
                    }
                    IdentifierKind::Type => {
                        return ast.err("Type is not a value");
                    }
                    _ => {}
                }
                Ok(vec![Statement {
                    kind: StatementKind::Value {
                        value: Value::Named(id.clone()),
                    },
                }])
            }
            AstKind::Array(a) => {
                let mut statements = vec![];
                let mut values = vec![];

                for ast in a {
                    let statement = self.ast_to_statements(ast, scope)?;
                    statements.extend(statement);

                    let last = statements.pop().expect("No statement");

                    match last.kind {
                        StatementKind::Value { value } => {
                            values.push(value);
                        }
                        StatementKind::CallAssign { assign, .. } => {
                            values.push(Value::Named(assign));
                        }
                        _ => return ast.err("Invalid argument"),
                    }
                }
                Ok(vec![Statement {
                    kind: StatementKind::Value {
                        value: Value::Constant(ConstantValue::Array(values)),
                    },
                }])
            }
        }
    }

    pub fn interpret(&mut self, ast: &Vec<Ast>) -> Result<()> {
        for ast in ast {
            self.ast_to_statements(ast, &self.scope.clone())?;
        }

        Ok(())
    }
}
