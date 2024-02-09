use anyhow::{anyhow, Result};

use crate::parser::ast::ParsedAst;

use super::{
    ast::{ItpAst, ItpAstKind},
    scope::Scope,
    value::{ItpFunctionParameters, ItpTypeValue, ItpValue, UnItpedFunctionValue},
    Interpreter,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type Macro = fn(&[ParsedAst], Rc<RefCell<Scope>>, &mut Interpreter) -> Result<Vec<ItpAst>>;

macro_rules! add_macro {
    ($macros:ident, $name:expr, $func:expr) => {
        $macros.insert($name.to_string(), $func);
    };
}

pub fn macros() -> HashMap<String, Macro> {
    let mut macros: HashMap<String, Macro> = HashMap::new();

    add_macro!(macros, "fn", fn_macro);
    add_macro!(macros, "set", set_macro);
    add_macro!(macros, "if", if_macro);
    add_macro!(macros, "while", while_macro);

    macros
}

/// (@fn name [arg1, arg2] (call arg1) (call arg2))
fn fn_macro(
    ast: &[ParsedAst],
    scope: Rc<RefCell<Scope>>,
    _itpr: &mut Interpreter,
) -> Result<Vec<ItpAst>> {
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

    // TODO: Interpret body when scope isn't the global scope
    let function = ItpValue::UnItpedFunction(UnItpedFunctionValue {
        name: name.name.clone(),
        parameters: ItpFunctionParameters {
            generics: vec![],
            parameters: args,
            variadic: false,
        },
        body: body.to_vec(),
        return_type: ItpTypeValue::Float,
    });

    let function = Rc::new(function);

    scope.borrow_mut().set(name.name, function)?;

    Ok(vec![])
}

/// (@set name value)
fn set_macro(
    ast: &[ParsedAst],
    scope: Rc<RefCell<Scope>>,
    itpr: &mut Interpreter,
) -> Result<Vec<ItpAst>> {
    if ast.len() != 2 {
        return Err(anyhow!("Expected 2 arguments"));
    }

    let name_ast = &ast.get(0).ok_or_else(|| anyhow!("Expected name"))?;
    let line = name_ast.line;
    let column = name_ast.column;
    let name = name_ast.as_identifier()?;
    let value = ast.get(1).ok_or_else(|| anyhow!("Expected value"))?;

    let value = itpr.interpret_ast(&value, &scope)?;

    if value.len() != 1 {
        return Err(anyhow!("Expected single value"));
    }

    scope.borrow_mut().set_or_replace(
        name.name.to_owned(),
        Rc::new(ItpValue::Named(name.clone(), value[0].get_type())),
    )?;

    Ok(vec![ItpAst {
        kind: ItpAstKind::SetVariable {
            name,
            value: Box::new(value[0].clone()),
        },
        line,
        column,
    }])
}

/// (@if condition then else)
fn if_macro(
    ast: &[ParsedAst],
    scope: Rc<RefCell<Scope>>,
    itpr: &mut Interpreter,
) -> Result<Vec<ItpAst>> {
    if ast.len() != 3 {
        return Err(anyhow!("Expected 3 arguments"));
    }

    let condition = ast.get(0).ok_or_else(|| anyhow!("Expected condition"))?;
    let line = condition.line;
    let column = condition.column;
    let then = ast.get(1).ok_or_else(|| anyhow!("Expected then"))?;
    let else_ = ast.get(2).ok_or_else(|| anyhow!("Expected else"))?;

    let condition = itpr.interpret_ast(&condition, &scope)?;
    let then = itpr.interpret_ast(&then, &scope)?;
    let else_ = itpr.interpret_ast(&else_, &scope)?;

    if condition.len() != 1 {
        return Err(anyhow!("Expected single condition"));
    }

    if then.len() != 1 {
        return Err(anyhow!("Expected single then"));
    }

    if else_.len() != 1 {
        return Err(anyhow!("Expected single else"));
    }

    Ok(vec![ItpAst {
        kind: ItpAstKind::Conditional {
            condition: Box::new(condition[0].clone()),
            then: Box::new(then[0].clone()),
            else_: Box::new(else_[0].clone()),
        },
        line,
        column,
    }])
}

/// (@while condition body)
fn while_macro(
    ast: &[ParsedAst],
    scope: Rc<RefCell<Scope>>,
    itpr: &mut Interpreter,
) -> Result<Vec<ItpAst>> {
    if ast.len() != 2 {
        return Err(anyhow!("Expected 2 arguments"));
    }

    let condition = ast.get(0).ok_or_else(|| anyhow!("Expected condition"))?;
    let line = condition.line;
    let column = condition.column;
    let body = ast.get(1).ok_or_else(|| anyhow!("Expected body"))?;

    let condition = itpr.interpret_ast(&condition, &scope)?;
    let body = itpr.interpret_ast(&body, &scope)?;

    if condition.len() != 1 {
        return Err(anyhow!("Expected single condition"));
    }

    if body.len() != 1 {
        return Err(anyhow!("Expected single body"));
    }

    Ok(vec![ItpAst {
        kind: ItpAstKind::Loop {
            condition: Box::new(condition[0].clone()),
            body: Box::new(body[0].clone()),
        },
        line,
        column,
    }])
}
