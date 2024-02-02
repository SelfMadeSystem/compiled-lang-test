use interpreter::Interpreter;
// use codegen::{compile_to_llvm_ir, run_ast};
// use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

// use parser::Parser;

// use crate::codegen::compile_to_file;

mod ast;
// mod codegen;
mod interpreter;
mod lexer;
mod parser;
mod tokens;

fn main() {
    let input = r#"
(@fn add_times_2[a, b]
    (* (+ a b) 2))

(@fn main[]
    (printf "Hello, world!\n")
    (printf "The answer is: %d\n" (add_times_2 19 23)))
"#;

    let tokens = Lexer::new(input.to_string()).lex().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast).unwrap();

    println!("{:#?}", interpreter);

    // let ir = compile_to_llvm_ir(&ast).expect("Unable to compile to LLVM IR");
    // println!("{}", ir);

    // compile_to_file(&ast, "sum").expect("Unable to compile to file");

    // let result = run_ast(&ast).expect("Unable to run AST");
    // println!("{} = {}", ast.to_str_expr(), result);
}
