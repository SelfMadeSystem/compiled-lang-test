use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::interpreter::{
    scope::Scope,
    statement::{Statement, StatementKind},
    value::{ItpConstantValue, ItpFunctionValue, ItpTypeValue, ItpValue},
    Interpreter,
};
use anyhow::{anyhow, Result};
use inkwell::module::Module;
use inkwell::values::FloatValue;
use inkwell::{
    builder::{Builder, BuilderError},
    types::FunctionType,
};
use inkwell::{
    context::Context,
    types::{BasicMetadataTypeEnum, BasicType},
    values::{BasicMetadataValueEnum, BasicValue, CallSiteValue},
};
use inkwell::{
    execution_engine::JitFunction,
    values::{AnyValueEnum, BasicValueEnum, GenericValue},
};
use inkwell::{AddressSpace, OptimizationLevel};

/// Convenience type alias for the `main` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type MainFunc = unsafe extern "C" fn() -> ();

pub struct CodeGen<'t> {
    context: &'t Context,
    module: Module<'t>,
    builder: Builder<'t>,
}

type CallResults<'a> = Rc<RefCell<HashMap<String, BasicMetadataValueEnum<'a>>>>;

impl<'t> CodeGen<'t> {
    pub fn new(context: &'t Context) -> Self {
        let module = context.create_module("main");
        let builder = context.create_builder();

        CodeGen {
            context,
            module,
            builder,
        }
    }

    fn declare_printf(&self) -> Result<(), BuilderError> {
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::from(0));
        let printf_type = self
            .context
            .void_type()
            .fn_type(&[i8_ptr_type.into()], true);

        self.module.add_function("printf", printf_type, None);

        Ok(())
    }

    fn jit_compile(&'t self, itp: &Interpreter) -> Option<JitFunction<MainFunc>> {
        let execution_engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .ok()?;

        self.compile(itp).ok()?;

        unsafe { execution_engine.get_function("main").ok() }
    }

    fn compile(&'t self, itp: &Interpreter) -> Result<()> {
        self.declare_printf()?;

        for (name, var) in itp.scope.borrow().bindings.iter() {
            let value = var.as_ref();
            let results: CallResults = Rc::new(RefCell::new(HashMap::new()));
            match value {
                ItpValue::Function(func) => {
                    let func_type = self.func_type(func);
                    let function = self.module.add_function(&name, func_type, None);
                    let basic_block = self.context.append_basic_block(function, "entry");

                    self.builder.position_at_end(basic_block);

                    for statement in &func.body {
                        self.statement(statement, results.clone())?;
                    }

                    self.builder.build_return(None)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn func_type(&self, func: &ItpFunctionValue) -> FunctionType<'_> {
        let mut types = vec![];

        for param in &func.parameters.parameters {
            types.push(self.type_of(&param.1));
        }

        let return_type = self.type_of(&func.return_type);

        self.context
            .f64_type()
            .fn_type(&types, false)
            .ptr_type(AddressSpace::default())
            .fn_type(&[return_type], false)
    }

    fn type_of(&self, param: &ItpTypeValue) -> BasicMetadataTypeEnum<'_> {
        match param {
            ItpTypeValue::Int => self.context.i64_type().as_basic_type_enum().into(),
            ItpTypeValue::Float => self.context.f64_type().as_basic_type_enum().into(),
            ItpTypeValue::String => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum()
                .into(),
            ItpTypeValue::Char => self.context.i8_type().as_basic_type_enum().into(),
            ItpTypeValue::Bool => self.context.bool_type().as_basic_type_enum().into(),
            ItpTypeValue::Array(_) => todo!(),
            ItpTypeValue::Function { .. } => todo!(),
        }
    }

    fn statement(&self, statement: &Statement, results: CallResults<'t>) -> Result<()> {
        match &statement.kind {
            StatementKind::Value { .. } => {}
            StatementKind::CallAssign {
                name,
                arguments,
                assign,
            } => {
                if self.native_function(statement, results.clone())? {
                    return Ok(());
                }
                let func = self.module.get_function(&name.name).ok_or_else(|| {
                    anyhow!(format!(
                        "Function {} not found.\n{:#?}",
                        name.name,
                        self.module.get_functions()
                    ))
                })?;
                let args = self.get_args(arguments, results.clone());
                let call = self.builder.build_call(func, &args, "call")?;
                let assign = assign.name.clone();

                results
                    .borrow_mut()
                    .insert(assign, call.try_as_basic_value().left().unwrap().into());
            }
            StatementKind::Return { .. } => todo!(),
        }

        Ok(())
    }

    fn get_args(
        &self,
        arguments: &[ItpValue],
        results: CallResults<'t>,
    ) -> Vec<BasicMetadataValueEnum<'t>> {
        arguments
            .iter()
            .map(|arg| self.get_arg(arg, results.clone()))
            .collect()
    }

    fn get_arg(&self, arg: &ItpValue, results: CallResults<'t>) -> BasicMetadataValueEnum<'t> {
        match arg {
            ItpValue::Constant(c) => self.get_constant(c),
            ItpValue::Named(name) => results.borrow().get(&name.name).map(|v| v).unwrap().clone(),
            _ => todo!(),
        }
    }

    fn get_constant(&self, c: &ItpConstantValue) -> BasicMetadataValueEnum<'t> {
        match c {
            ItpConstantValue::Int(i) => self.context.i64_type().const_int(*i as u64, false).into(),
            ItpConstantValue::Float(f) => self.context.f64_type().const_float(*f).into(),
            ItpConstantValue::String(_) => todo!(),
            ItpConstantValue::Char(c) => self.context.i8_type().const_int(*c as u64, false).into(),
            ItpConstantValue::Bool(b) => {
                self.context.bool_type().const_int(*b as u64, false).into()
            }
            ItpConstantValue::Array(a) => {
                let mut values = vec![];

                for value in a {
                    match value {
                        ItpValue::Constant(c) => {
                            values.push(self.get_constant(c).into_pointer_value())
                        }
                        _ => todo!(),
                    }
                }

                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .const_array(&values)
                    .into()
            }
            ItpConstantValue::None => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .const_null()
                .into(),
        }
    }

    /// stuff like `+`, `-`, `*`, `/`, etc.
    fn native_function(&self, statement: &Statement, results: CallResults<'t>) -> Result<bool> {
        let StatementKind::CallAssign {
            name,
            arguments,
            assign,
        } = &statement.kind
        else {
            return Ok(false);
        };

        let name = name.name.clone();
        let assign = assign.name.clone();

        let args = self.get_args(&arguments, results.clone());

        match name.as_str() {
            "+" => {
                let add = self.builder.build_float_add(
                    args[0].into_float_value(),
                    args[1].into_float_value(),
                    "add",
                )?;

                results.borrow_mut().insert(assign, add.as_basic_value_enum().into());
            }
            "-" => {
                let sub = self.builder.build_float_sub(
                    args[0].into_float_value(),
                    args[1].into_float_value(),
                    "sub",
                )?;

                results.borrow_mut().insert(assign, sub.as_basic_value_enum().into());
            }
            "*" => {
                let mul = self.builder.build_float_mul(
                    args[0].into_float_value(),
                    args[1].into_float_value(),
                    "mul",
                )?;

                results.borrow_mut().insert(assign, mul.as_basic_value_enum().into());
            }
            "/" => {
                let div = self.builder.build_float_div(
                    args[0].into_float_value(),
                    args[1].into_float_value(),
                    "div",
                )?;

                results.borrow_mut().insert(assign, div.as_basic_value_enum().into());
            }
            _ => return Ok(false),
        }

        Ok(true)
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
