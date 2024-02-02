use super::ast::{Ast, AstKind};
use anyhow::{anyhow, Result};
use inkwell::builder::{Builder, BuilderError};
use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use inkwell::module::Module;
use inkwell::values::FloatValue;
use inkwell::{AddressSpace, OptimizationLevel};

/// Convenience type alias for the `ast` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type AstFunc = unsafe extern "C" fn() -> f64;

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    fn jit_compile_ast(&self, ast: &Vec<Ast>) -> Result<JitFunction<AstFunc>> {
        let execution_engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| anyhow!(e.to_string()))?;

        self.compile_ast(&ast)?;

        unsafe {
            execution_engine
                .get_function("main")
                .map_err(|e| anyhow!(e.to_string()))
        }
    }

    fn compile_ast(&self, ast: &Vec<Ast>) -> Result<()> {
        let f64_type = self.context.f64_type();
        let fn_type = f64_type.fn_type(&[], false);
        let function = self.module.add_function("ast", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let result = self.compile_ast_node(ast)?;

        self.builder.build_return(Some(&result))?;

        Ok(())
    }

    /* fn declare_printf(&self) -> Result<(), BuilderError> {
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::from(0));
        let printf_type = self
            .context
            .void_type()
            .fn_type(&[i8_ptr_type.into()], true);

        self.module.add_function("printf", printf_type, None);

        Ok(())
    } */

    fn compile_ast_node(&self, ast: &Ast) -> Result<FloatValue, BuilderError> {
        match &ast.kind {
            AstKind::Number(n) => Ok(self.context.f64_type().const_float(*n)),
            AstKind::Input => {
                let f64_type = self.context.f64_type();

                let format_string = self.builder.build_global_string_ptr("%lf\0", "fmt")?;
                let input_ptr = self.builder.build_alloca(f64_type, "input")?;

                let scanf_fn = self
                    .module
                    .get_function("scanf")
                    .ok_or_else(|| BuilderError::GEPIndex)?;

                self.builder.build_call(
                    scanf_fn,
                    &[format_string.as_pointer_value().into(), input_ptr.into()],
                    "scanf",
                )?;

                let input_value = self.builder.build_load(input_ptr, "input_val")?;
                Ok(input_value.into_float_value())
            }
            AstKind::BinaryOp { op, lhs, rhs } => {
                let lhs = self.compile_ast_node(lhs)?;
                let rhs = self.compile_ast_node(rhs)?;

                match op {
                    BinaryOp::Add => self.builder.build_float_add(lhs, rhs, "addtmp"),
                    BinaryOp::Sub => self.builder.build_float_sub(lhs, rhs, "subtmp"),
                    BinaryOp::Mul => self.builder.build_float_mul(lhs, rhs, "multmp"),
                    BinaryOp::Div => self.builder.build_float_div(lhs, rhs, "divtmp"),
                }
            }
        }
    }
}

pub fn run_ast(ast: &Ast) -> Result<f64> {
    let context = Context::create();
    let module = context.create_module("sum");
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
    };

    let sum = codegen
        .jit_compile_ast(&ast)
        .ok_or(anyhow!("Unable to JIT compile `sum`"))?;

    unsafe {
        let result = sum.call();
        Ok(result)
    }
}

pub fn compile_to_llvm_ir(ast: &Vec<Ast>) -> Result<String> {
    let context = Context::create();
    let module = context.create_module("sum");
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
    };

    codegen.compile_ast(&ast)?;

    Ok(codegen.module.print_to_string().to_string())
}

pub fn compile_to_file(ast: &Ast, filename: &str) -> Result<()> {
    let context = Context::create();
    let module = context.create_module("sum");
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
    };

    codegen.compile_ast(&ast)?;

    if codegen
        .module
        .write_bitcode_to_path(std::path::Path::new(filename))
    {
        Ok(())
    } else {
        Err(anyhow!("Unable to write bitcode to file"))
    }
}
