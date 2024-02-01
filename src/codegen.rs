use anyhow::{anyhow, Result};
use super::ast::{Ast, AstKind, BinaryOp};
use inkwell::builder::{Builder, BuilderError};
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::values::FloatValue;
use inkwell::OptimizationLevel;

/// Convenience type alias for the `ast` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type AstFunc = unsafe extern "C" fn() -> f64;

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    fn jit_compile_ast(&self, ast: &Ast) -> Option<JitFunction<AstFunc>> {
        let f64_type = self.context.f64_type();
        let fn_type = f64_type.fn_type(&[], false);
        let function = self.module.add_function("ast", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let result = self.compile_ast(ast);

        match result {
            Ok(result) => {
                if let Err(err) = self.builder.build_return(Some(&result)) {
                    eprintln!("Error compiling AST: {:?}", err);
                    return None;
                }
            }
            Err(err) => {
                eprintln!("Error compiling AST: {:?}", err);
                return None;
            }
        }

        unsafe { self.execution_engine.get_function("ast").ok() }
    }

    fn compile_ast(&self, ast: &Ast) -> Result<FloatValue, BuilderError> {
        match &ast.kind {
            AstKind::Number(n) => Ok(self.context.f64_type().const_float(*n)),
            AstKind::BinaryOp { op, lhs, rhs } => {
                let lhs = self.compile_ast(lhs)?;
                let rhs = self.compile_ast(rhs)?;

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

pub fn run_ast(ast: Ast) -> Result<()> {
    let context = Context::create();
    let module = context.create_module("sum");
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .or_else(|err| Err(anyhow!("LLVM error: {:?}", err)))?;
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
    };

    let sum = codegen
        .jit_compile_ast(&ast)
        .ok_or(anyhow!("Unable to JIT compile `sum`"))?;

    unsafe {
        println!("{} = {}", ast.to_str_expr(), sum.call());
    }

    Ok(())
}
