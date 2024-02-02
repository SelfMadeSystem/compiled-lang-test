use crate::tokens::Identifier;

use super::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    // Call {
    //     name: Identifier,
    //     arguments: Vec<Value>,
    // },
    CallAssign {
        name: Identifier,
        arguments: Vec<Value>,
        assign: Identifier,
    },
    Value {
        value: Value,
    },
    // Jump {
    //     target: String,
    // },
    // JumpIf {
    //     condition: Value,
    //     then_target: String,
    //     else_target: String,
    // },
    Return {
        value: Value,
    },
}
