use anyhow::{anyhow, Result};
use inkwell::values::AnyValueEnum;

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
        _ => Ok(None),
    }
}
