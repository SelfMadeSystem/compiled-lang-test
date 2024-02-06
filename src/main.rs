use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

use crate::codegen::{compile_to_llvm_ir, run_jit};

mod codegen;
mod interpreter;
mod lexer;
mod parser;
mod tokens;

fn main() {
    let input = r#"
(@fn main[]
    (printf "(1 + 2) * 3 = %f\n\0" (* (+ 1.0 2.0) 3.0)))
"#;

    let tokens = Lexer::new(input.to_string()).lex().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast).unwrap();

    println!("=== LLVM IR ===");
    let ir = compile_to_llvm_ir(&interpreter).unwrap();
    println!("{}", ir);
    println!("=== Running JIT ===");
    run_jit(&interpreter).unwrap();
}
