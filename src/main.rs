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
(@fn add[a, b, c]
    (@set d (+ a b))
    (@set e (+ d c))
    e)

(@fn main[]
    (@set c 6.0)
    (@set c (add c 4.0 0.0))
    (@set c (add c 5.0 0.0))
    (printf "(1 + 2 + 3) * %f = %f\n\0" c (* (add 1.0 2.0 3.0) c)))
"#;

    let tokens = Lexer::new(input.to_string()).lex().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast).unwrap();

    println!("{:#?}", interpreter);

    println!("=== LLVM IR ===");
    let ir = compile_to_llvm_ir(&interpreter).unwrap();
    println!("{}", ir);
    println!("=== Running JIT ===");
    run_jit(&interpreter).unwrap();
}
