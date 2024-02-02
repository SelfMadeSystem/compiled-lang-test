use anyhow::Result;

use super::{
    scope::Scope,
    statement::Statement,
    value::{ItpFunctionParameters, ItpFunctionValue, ItpTypeValue, ItpValue},
    Interpreter,
};
use crate::ast::Ast;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type Macro = fn(&[Ast], &mut Interpreter) -> Result<Vec<Statement>>;

macro_rules! add_macro {
    ($macros:ident, $name:expr, $func:expr) => {
        $macros.insert($name.to_string(), $func);
    };
}

pub fn macros() -> HashMap<String, Macro> {
    let mut macros: HashMap<String, Macro> = HashMap::new();

    add_macro!(macros, "fn", fn_macro);

    macros
}

/// (@fn name [arg1, arg2] (call arg1) (call arg2))
fn fn_macro(ast: &[Ast], itpr: &mut Interpreter) -> Result<Vec<Statement>> {
    let name = ast[0].as_identifier()?;
    let args = ast[1].as_array()?;
    let body = &ast[2..];

    let mut statements = vec![];

    let new_scope = Scope::new_child(itpr.scope.clone());
    let new_scope = Rc::new(RefCell::new(new_scope));

    for ast in body {
        let statement = itpr.ast_to_statements(ast, &new_scope)?;
        statements.extend(statement);
    }

    let mut args = args
        .iter()
        .map(|arg| {
            arg.as_identifier()
                .map(|id| (id.name.clone(), ItpTypeValue::Float))
        })
        .collect::<Result<Vec<(String, ItpTypeValue)>>>()?;

    let function = ItpValue::Function(ItpFunctionValue {
        name: name.name.clone(),
        parameters: ItpFunctionParameters {
            parameters: args,
            variadic: false,
        },
        body: statements,
        return_type: ItpTypeValue::Float,
    });

    let function = Rc::new(function);

    itpr.scope.borrow_mut().set(name.name, function)?;

    Ok(vec![])
}
