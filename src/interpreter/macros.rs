use anyhow::Result;

use crate::parser::ast::ParsedAst;

use super::{
    ast::ItpAst,
    scope::Scope,
    value::{
        ItpFunctionParameters, ItpFunctionValue, ItpTypeValue, ItpValue, UnItpedFunctionValue,
    },
    Interpreter,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type Macro = fn(&[ParsedAst], &mut Interpreter) -> Result<Vec<ItpAst>>;

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
fn fn_macro(ast: &[ParsedAst], itpr: &mut Interpreter) -> Result<Vec<ItpAst>> {
    let name = ast[0].as_identifier()?;
    let args = ast[1].as_array()?;
    let body = &ast[2..];

    let args = args
        .iter()
        .map(|arg| {
            arg.as_identifier()
                .map(|id| (id.name.clone(), ItpTypeValue::Float))
        })
        .collect::<Result<Vec<(String, ItpTypeValue)>>>()?;

    let function = ItpValue::UnItpedFunction(UnItpedFunctionValue {
        name: name.name.clone(),
        parameters: ItpFunctionParameters {
            parameters: args,
            variadic: false,
        },
        body: body.to_vec(),
        return_type: ItpTypeValue::Float,
    });

    let function = Rc::new(function);

    itpr.scope.borrow_mut().set(name.name, function)?;

    Ok(vec![])
}
