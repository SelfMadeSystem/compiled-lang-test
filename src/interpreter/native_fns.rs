use std::rc::Rc;

use super::{
    value::{ItpFunctionParameters, ItpTypeValue, ItpValue::NativeFunction, NativeFunctionValue},
    Interpreter,
};

macro_rules! add_native_fn {
    ($scope:expr, $name:expr, $parameters:expr, $return_type:expr, $intrinsic:expr, ) => {{
        $scope
            .set(
                $name.to_string(),
                Rc::new(NativeFunction(NativeFunctionValue {
                    name: $name.to_string(),
                    parameters: $parameters,
                    return_type: $return_type,
                    intrinsic: $intrinsic,
                })),
            )
            .unwrap();
    }};
}

pub fn add_native_fns(itp: &mut Interpreter) {
    let mut scope = itp.scope.borrow_mut();

    add_native_fn!(
        scope,
        "+",
        ItpFunctionParameters {
            parameters: vec![
                ("a".to_string(), ItpTypeValue::Float),
                ("b".to_string(), ItpTypeValue::Float)
            ],
            variadic: false,
        },
        ItpTypeValue::Float,
        true,
    );

    add_native_fn!(
        scope,
        "-",
        ItpFunctionParameters {
            parameters: vec![
                ("a".to_string(), ItpTypeValue::Float),
                ("b".to_string(), ItpTypeValue::Float)
            ],
            variadic: false,
        },
        ItpTypeValue::Float,
        true,
    );

    add_native_fn!(
        scope,
        "*",
        ItpFunctionParameters {
            parameters: vec![
                ("a".to_string(), ItpTypeValue::Float),
                ("b".to_string(), ItpTypeValue::Float)
            ],
            variadic: false,
        },
        ItpTypeValue::Float,
        true,
    );

    add_native_fn!(
        scope,
        "/",
        ItpFunctionParameters {
            parameters: vec![
                ("a".to_string(), ItpTypeValue::Float),
                ("b".to_string(), ItpTypeValue::Float)
            ],
            variadic: false,
        },
        ItpTypeValue::Float,
        true,
    );

    add_native_fn!(
        scope,
        "==",
        ItpFunctionParameters {
            parameters: vec![
                ("a".to_string(), ItpTypeValue::Float),
                ("b".to_string(), ItpTypeValue::Float)
            ],
            variadic: false,
        },
        ItpTypeValue::Bool,
        true,
    );

    add_native_fn!(
        scope,
        "printf",
        ItpFunctionParameters {
            parameters: vec![("format".to_string(), ItpTypeValue::String)],
            variadic: true,
        },
        ItpTypeValue::Void,
        false,
    );
}
