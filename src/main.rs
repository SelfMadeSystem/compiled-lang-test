use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

use crate::codegen::{compile_to_file, compile_to_llvm_ir, run_jit};

mod codegen;
mod interpreter;
mod lexer;
mod parser;
mod tokens;

fn main() {
    let input = r#"
(@fn fib[a]
    (@if (== a 0)
        0
        (@if (== a 1)
            1
            (+ (fib (- a 1)) (fib (- a 2))))))

(@fn main[]
    (@set n 10)
    (printf "fib(%f) = %f\n" n (fib n))
    1)
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
    println!("=== Writing to file ===");
    compile_to_file(&interpreter, "hello").unwrap();
    println!("=== Running JIT ===");
    run_jit(&interpreter).unwrap();
}
