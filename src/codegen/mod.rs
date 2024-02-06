use std::{cell::RefCell, collections::HashMap};

use crate::interpreter::{
    ast::{ItpAst, ItpAstKind},
    value::{BaseFunctionValue, ItpConstantValue, ItpTypeValue, ItpValue, ToBaseFunctionValue},
    Interpreter,
};
use anyhow::{anyhow, Result};
use inkwell::{types::BasicTypeEnum, values::{FunctionValue, PointerValue}};
use inkwell::{builder::Builder, types::FunctionType};
use inkwell::{
    context::Context,
    types::{BasicMetadataTypeEnum, BasicType},
    values::{BasicMetadataValueEnum, BasicValue},
};
use inkwell::{
    execution_engine::JitFunction,
    values::{AnyValueEnum, BasicValueEnum},
};
use inkwell::{
    module::Module,
    types::{AnyType, AnyTypeEnum},
    values::AnyValue,
};
use inkwell::{AddressSpace, OptimizationLevel};

use self::intrinsic_fns::check_intrinsic_fn;

mod intrinsic_fns;

/// Convenience type alias for the `main` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type MainFunc = unsafe extern "C" fn() -> ();

fn try_as_basic_metadata_value_enum<'a>(
    value: AnyValueEnum<'a>,
) -> Result<BasicMetadataValueEnum<'a>> {
    match value {
        AnyValueEnum::ArrayValue(a) => Ok(a.as_basic_value_enum().into()),
        AnyValueEnum::IntValue(i) => Ok(i.as_basic_value_enum().into()),
        AnyValueEnum::FloatValue(f) => Ok(f.as_basic_value_enum().into()),
        AnyValueEnum::PointerValue(p) => Ok(p.as_basic_value_enum().into()),
        AnyValueEnum::StructValue(s) => Ok(s.as_basic_value_enum().into()),
        AnyValueEnum::VectorValue(v) => Ok(v.as_basic_value_enum().into()),
        AnyValueEnum::MetadataValue(m) => Ok(BasicMetadataValueEnum::MetadataValue(m)),
        _ => Err(anyhow!("Value is not a basic metadata value")),
    }
}

fn try_as_basic_metadata_type_enum<'a>(ty: AnyTypeEnum<'a>) -> Result<BasicMetadataTypeEnum<'a>> {
    match ty {
        AnyTypeEnum::ArrayType(t) => Ok(t.as_basic_type_enum().into()),
        AnyTypeEnum::FloatType(t) => Ok(t.as_basic_type_enum().into()),
        AnyTypeEnum::IntType(t) => Ok(t.as_basic_type_enum().into()),
        AnyTypeEnum::PointerType(t) => Ok(t.as_basic_type_enum().into()),
        AnyTypeEnum::StructType(t) => Ok(t.as_basic_type_enum().into()),
        AnyTypeEnum::VectorType(t) => Ok(t.as_basic_type_enum().into()),
        AnyTypeEnum::VoidType(_) => Err(anyhow!("Void type is not a basic metadata type")),
        _ => Err(anyhow!("Type is not a basic metadata type")),
    }
}

fn try_as_basic_type_enum<'a>(ty: AnyTypeEnum<'a>) -> Result<BasicTypeEnum<'a>> {
    match ty {
        AnyTypeEnum::ArrayType(t) => Ok(t.as_basic_type_enum()),
        AnyTypeEnum::FloatType(t) => Ok(t.as_basic_type_enum()),
        AnyTypeEnum::IntType(t) => Ok(t.as_basic_type_enum()),
        AnyTypeEnum::PointerType(t) => Ok(t.as_basic_type_enum()),
        AnyTypeEnum::StructType(t) => Ok(t.as_basic_type_enum()),
        AnyTypeEnum::VectorType(t) => Ok(t.as_basic_type_enum()),
        AnyTypeEnum::VoidType(t) => Err(anyhow!("Void type is not a basic type")),
        AnyTypeEnum::FunctionType(_) => Err(anyhow!("Function type is not a basic type")),
    }
}

fn try_as_basic_value_enum<'a>(value: AnyValueEnum<'a>) -> Result<BasicValueEnum<'a>> {
    match value {
        AnyValueEnum::ArrayValue(a) => Ok(a.as_basic_value_enum()),
        AnyValueEnum::IntValue(i) => Ok(i.as_basic_value_enum()),
        AnyValueEnum::FloatValue(f) => Ok(f.as_basic_value_enum()),
        AnyValueEnum::PointerValue(p) => Ok(p.as_basic_value_enum()),
        AnyValueEnum::StructValue(s) => Ok(s.as_basic_value_enum()),
        AnyValueEnum::VectorValue(v) => Ok(v.as_basic_value_enum()),
        AnyValueEnum::MetadataValue(_) => Err(anyhow!("Metadata value is not a basic value")),
        _ => Err(anyhow!("Value is not a basic value")),
    }
}

pub struct CodeGen<'t> {
    context: &'t Context,
    module: Module<'t>,
    builder: Builder<'t>,
    variables: RefCell<HashMap<String, PointerValue<'t>>>,
}

impl<'t> CodeGen<'t> {
    pub fn new(context: &'t Context) -> Self {
        let module = context.create_module("main");
        let builder = context.create_builder();

        CodeGen {
            context,
            module,
            builder,
            variables: RefCell::new(HashMap::new()),
        }
    }

    // fn declare_printf(&self) -> Result<(), BuilderError> {
    //     let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::from(0));
    //     let printf_type = self
    //         .context
    //         .void_type()
    //         .fn_type(&[i8_ptr_type.into()], true);

    //     self.module.add_function("printf", printf_type, None);

    //     Ok(())
    // }

    fn jit_compile(&'t self, itp: &Interpreter) -> Result<JitFunction<MainFunc>> {
        let execution_engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|err| anyhow!(format!("{}", err)))?;

        self.compile(itp).map_err(|err| anyhow!(err))?;

        unsafe {
            execution_engine
                .get_function("main")
                .map_err(|err| anyhow!(err))
        }
    }

    fn compile(&'t self, itp: &Interpreter) -> Result<()> {
        self.declare_functions(itp)?;
        self.compile_functions(itp)?;

        Ok(())
    }

    fn declare_functions(&'t self, itp: &Interpreter) -> Result<()> {
        for (name, var) in itp.scope.borrow().bindings.iter() {
            let value = var.as_ref();
            match value {
                ItpValue::Function(func) => {
                    let func_type = self.func_type(&func.to_base())?;
                    self.module.add_function(&name, func_type, None);
                }
                ItpValue::NativeFunction(func) => {
                    if func.intrinsic {
                        continue;
                    }
                    let func_type = self.func_type(&func.to_base())?;
                    self.module.add_function(&name, func_type, None);
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn compile_functions(&'t self, itp: &Interpreter) -> Result<()> {
        for (name, var) in itp.scope.borrow().bindings.iter() {
            let value = var.as_ref();
            match value {
                ItpValue::Function(func) => {
                    self.variables.borrow_mut().clear();

                    let function = self
                        .module
                        .get_function(&name)
                        .expect("Function not found???");

                    let basic_block = self.context.append_basic_block(function, "entry");

                    self.builder.position_at_end(basic_block);

                    let mut last_value = None;
                    for statement in &func.body {
                        last_value = Some(self.ast(statement, &function)?);
                    }

                    if let Some(last_value) = last_value {
                        match last_value {
                            AnyValueEnum::IntValue(v) => {
                                self.builder.build_return(Some(&v))?;
                            }
                            AnyValueEnum::FloatValue(v) => {
                                self.builder.build_return(Some(&v))?;
                            }
                            AnyValueEnum::PointerValue(v) => {
                                self.builder.build_return(Some(&v))?;
                            }
                            AnyValueEnum::StructValue(v) => {
                                self.builder.build_return(Some(&v))?;
                            }
                            AnyValueEnum::VectorValue(v) => {
                                self.builder.build_return(Some(&v))?;
                            }
                            AnyValueEnum::ArrayValue(v) => {
                                self.builder.build_return(Some(&v))?;
                            }
                            AnyValueEnum::InstructionValue(_) => {
                                self.builder.build_return(None)?;
                            }
                            AnyValueEnum::PhiValue(v) => {
                                self.builder.build_return(Some(&v.as_basic_value()))?;
                            }
                            AnyValueEnum::FunctionValue(_) => todo!(),
                            AnyValueEnum::MetadataValue(_) => todo!(),
                        }
                    } else {
                        self.builder.build_return(None)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn func_type(&self, func: &BaseFunctionValue) -> Result<FunctionType<'_>> {
        let mut types = vec![];
        let is_var_args = func.parameters.variadic;

        for param in &func.parameters.parameters {
            types.push(try_as_basic_metadata_type_enum(self.type_of(&param.1))?);
        }

        let return_type = self.type_of(&func.return_type);

        Ok(match return_type {
            AnyTypeEnum::ArrayType(t) => t.fn_type(&types, is_var_args),
            AnyTypeEnum::FloatType(t) => t.fn_type(&types, is_var_args),
            AnyTypeEnum::IntType(t) => t.fn_type(&types, is_var_args),
            AnyTypeEnum::PointerType(t) => t.fn_type(&types, is_var_args),
            AnyTypeEnum::StructType(t) => t.fn_type(&types, is_var_args),
            AnyTypeEnum::VectorType(t) => t.fn_type(&types, is_var_args),
            AnyTypeEnum::VoidType(t) => t.fn_type(&types, is_var_args),
            AnyTypeEnum::FunctionType(_) => Err(anyhow!("Function type not allowed"))?,
        })
    }

    fn type_of(&self, param: &ItpTypeValue) -> AnyTypeEnum<'_> {
        match param {
            ItpTypeValue::Int => self.context.i64_type().as_any_type_enum(),
            ItpTypeValue::Float => self.context.f64_type().as_any_type_enum(),
            ItpTypeValue::String => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .as_any_type_enum(),
            ItpTypeValue::Char => self.context.i8_type().as_any_type_enum(),
            ItpTypeValue::Bool => self.context.bool_type().as_any_type_enum(),
            ItpTypeValue::Array(_) => todo!(),
            ItpTypeValue::Function { .. } => todo!(),
            ItpTypeValue::Void => self.context.void_type().as_any_type_enum(),
        }
    }

    fn get_constant(&self, c: &ItpConstantValue) -> Result<BasicValueEnum<'t>> {
        match c {
            ItpConstantValue::Int(i) => {
                Ok(self.context.i64_type().const_int(*i as u64, false).into())
            }
            ItpConstantValue::Float(f) => Ok(self.context.f64_type().const_float(*f).into()),
            ItpConstantValue::String(s) => Ok(self
                .builder
                .build_global_string_ptr(&s, "str")?
                .as_pointer_value()
                .as_basic_value_enum()),
            ItpConstantValue::Char(c) => {
                Ok(self.context.i8_type().const_int(*c as u64, false).into())
            }
            ItpConstantValue::Bool(b) => {
                Ok(self.context.bool_type().const_int(*b as u64, false).into())
            }
            ItpConstantValue::Array(_) => todo!(),
        }
    }

    fn ast(&self, ast: &ItpAst, func: &FunctionValue<'t>) -> Result<AnyValueEnum<'t>> {
        match &ast.kind {
            ItpAstKind::Constant(c) => Ok(self.get_constant(&c)?.as_any_value_enum()),
            ItpAstKind::Variable { name, .. } => {
                let vars = self.variables.borrow();
                let value = vars.get(&name.name).ok_or_else(|| {
                    anyhow!("Variable {} not found in function {}", name.name, func.get_name().to_str().unwrap())
                })?;
                let value = self.builder.build_load(*value, "load")
                    .map_err(|err| anyhow!(err))?;
                Ok(value.as_any_value_enum())
            },
            ItpAstKind::SetVariable { name, value } => {
                let value = self.ast(value, func)?;
                if self.variables.borrow().contains_key(&name.name) {
                    let b = self.variables.borrow();
                    let var = b.get(&name.name).unwrap();
                    self.builder.build_store(*var, try_as_basic_value_enum(value)?)
                        .map_err(|err| anyhow!(err))?;
                } else {
                    let alloca = self.builder.build_alloca(try_as_basic_type_enum(value.get_type())?, &name.name)
                        .map_err(|err| anyhow!(err))?;
                    self.builder.build_store(alloca, try_as_basic_value_enum(value)?)
                        .map_err(|err| anyhow!(err))?;
                    self.variables.borrow_mut().insert(name.name.clone(), alloca);
                }
                Ok(value)
            }
            ItpAstKind::Param { position, .. } => {
                let arg = func.get_nth_param(*position).ok_or_else(|| {
                    anyhow!(
                        "Parameter {} not found in function {}",
                        position,
                        func.get_name().to_str().unwrap()
                    )
                })?;
                Ok(arg.as_any_value_enum())
            }
            ItpAstKind::Call {
                function,
                arguments,
                ..
            } => {
                let mut args = vec![];

                for arg in arguments {
                    args.push(self.ast(arg, func)?);
                }

                Ok(self.call(function.name.clone(), &args)?)
            }
        }
    }

    fn call(&self, name: String, args: &[AnyValueEnum<'t>]) -> Result<AnyValueEnum<'t>> {
        if let Some(v) = check_intrinsic_fn(&name, self, args)? {
            return Ok(v);
        }

        let function = self
            .module
            .get_function(&name)
            .ok_or_else(|| anyhow!("Function '{}' not found in the module", name))?;

        let args = args
            .iter()
            .map(|arg| try_as_basic_metadata_value_enum(*arg))
            .collect::<Result<Vec<BasicMetadataValueEnum>>>()?;

        let result = self
            .builder
            .build_call(function, &args, "call")
            .map_err(|err| anyhow!(err))?;

        Ok(result.as_any_value_enum())
    }
}

pub fn compile_to_llvm_ir(itp: &Interpreter) -> Result<String> {
    let context = Context::create();
    let codegen = (&CodeGen::new(&context)) as *const CodeGen<'_>;

    unsafe {
        (*codegen).compile(itp)?;
    }

    let ir = unsafe { &*codegen }.module.print_to_string().to_string();

    Ok(ir)
}

pub fn run_jit(itp: &Interpreter) -> Result<()> {
    let context = Context::create();
    let codegen = (&CodeGen::new(&context)) as *const CodeGen<'_>;

    unsafe {
        (*codegen).jit_compile(itp)?.call();
    };

    Ok(())
}
