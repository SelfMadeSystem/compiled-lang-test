use anyhow::{anyhow, Result};
use inkwell::{
    values::{AnyValue, AnyValueEnum, AsValueRef, PointerValue},
    AddressSpace,
};

use super::CodeGen;

pub(crate) fn check_intrinsic_fn<'a>(
    name: &str,
    codegen: &CodeGen<'a>,
    params: &[AnyValueEnum<'a>],
) -> Result<Option<AnyValueEnum<'a>>> {
    match name {
        "+" => {
            let lhs = params
                .get(0)
                .ok_or_else(|| anyhow!("Expected first parameter for '+' function"))?;
            let rhs = params
                .get(1)
                .ok_or_else(|| anyhow!("Expected second parameter for '+' function"))?;

            let AnyValueEnum::FloatValue(lhs) = lhs else {
                return Err(anyhow!(
                    "Expected float for first parameter of '+' function"
                ));
            };
            let AnyValueEnum::FloatValue(rhs) = rhs else {
                return Err(anyhow!(
                    "Expected float for second parameter of '+' function"
                ));
            };

            let result = codegen.builder.build_float_add(*lhs, *rhs, "addtmp");

            match result {
                Ok(result) => Ok(Some(AnyValueEnum::FloatValue(result))),
                Err(err) => Err(anyhow!(err)),
            }
        }
        "-" => {
            let lhs = params
                .get(0)
                .ok_or_else(|| anyhow!("Expected first parameter for '-' function"))?;
            let rhs = params
                .get(1)
                .ok_or_else(|| anyhow!("Expected second parameter for '-' function"))?;

            let AnyValueEnum::FloatValue(lhs) = lhs else {
                return Err(anyhow!(
                    "Expected float for first parameter of '-' function"
                ));
            };
            let AnyValueEnum::FloatValue(rhs) = rhs else {
                return Err(anyhow!(
                    "Expected float for second parameter of '-' function"
                ));
            };

            let result = codegen.builder.build_float_sub(*lhs, *rhs, "subtmp");

            match result {
                Ok(result) => Ok(Some(AnyValueEnum::FloatValue(result))),
                Err(err) => Err(anyhow!(err)),
            }
        }
        "*" => {
            let lhs = params
                .get(0)
                .ok_or_else(|| anyhow!("Expected first parameter for '*' function"))?;
            let rhs = params
                .get(1)
                .ok_or_else(|| anyhow!("Expected second parameter for '*' function"))?;

            let AnyValueEnum::FloatValue(lhs) = lhs else {
                return Err(anyhow!(
                    "Expected float for first parameter of '*' function"
                ));
            };
            let AnyValueEnum::FloatValue(rhs) = rhs else {
                return Err(anyhow!(
                    "Expected float for second parameter of '*' function"
                ));
            };

            let result = codegen.builder.build_float_mul(*lhs, *rhs, "multmp");

            match result {
                Ok(result) => Ok(Some(AnyValueEnum::FloatValue(result))),
                Err(err) => Err(anyhow!(err)),
            }
        }
        "/" => {
            let lhs = params
                .get(0)
                .ok_or_else(|| anyhow!("Expected first parameter for '/' function"))?;
            let rhs = params
                .get(1)
                .ok_or_else(|| anyhow!("Expected second parameter for '/' function"))?;

            let AnyValueEnum::FloatValue(lhs) = lhs else {
                return Err(anyhow!(
                    "Expected float for first parameter of '/' function"
                ));
            };
            let AnyValueEnum::FloatValue(rhs) = rhs else {
                return Err(anyhow!(
                    "Expected float for second parameter of '/' function"
                ));
            };

            let result = codegen.builder.build_float_div(*lhs, *rhs, "divtmp");

            match result {
                Ok(result) => Ok(Some(AnyValueEnum::FloatValue(result))),
                Err(err) => Err(anyhow!(err)),
            }
        }
        "==" => {
            let lhs = params
                .get(0)
                .ok_or_else(|| anyhow!("Expected first parameter for '==' function"))?;
            let rhs = params
                .get(1)
                .ok_or_else(|| anyhow!("Expected second parameter for '==' function"))?;

            let AnyValueEnum::FloatValue(lhs) = lhs else {
                return Err(anyhow!(
                    "Expected float for first parameter of '==' function"
                ));
            };
            let AnyValueEnum::FloatValue(rhs) = rhs else {
                return Err(anyhow!(
                    "Expected float for second parameter of '==' function"
                ));
            };

            let result = codegen.builder.build_float_compare(
                inkwell::FloatPredicate::OEQ,
                *lhs,
                *rhs,
                "eqtmp",
            );

            match result {
                Ok(result) => Ok(Some(AnyValueEnum::IntValue(result))),
                Err(err) => Err(anyhow!(err)),
            }
        }
        "get" => {
            let array = params
                .get(0)
                .ok_or_else(|| anyhow!("Expected first parameter for 'get' function"))?;
            let index = params
                .get(1)
                .ok_or_else(|| anyhow!("Expected second parameter for 'get' function"))?;

            match array {
                AnyValueEnum::PointerValue(array) => {
                    let index = match index {
                        AnyValueEnum::FloatValue(index) => {
                            codegen.builder.build_float_to_unsigned_int(
                                *index,
                                codegen.context.i64_type(),
                                "index",
                            )?
                        }
                        AnyValueEnum::IntValue(index) => *index,
                        _ => return Err(anyhow!("Expected number for index of 'get' function")),
                    };

                    let result = unsafe {
                        codegen.builder.build_gep(
                            *array,
                            &[codegen.context.i64_type().const_zero(), index],
                            "elementptr",
                        )
                    }?;

                    let result = codegen.builder.build_load(result, "element")?;

                    Ok(Some(result.as_any_value_enum()))
                }
                a => Err(anyhow!(
                    "Expected pointer for first parameter of 'get' function. Got {:?}",
                    a
                )),
            }
        }
        _ => Ok(None),
    }
}
