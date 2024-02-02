use codegen::compile_to_llvm_ir;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

mod ast;
mod codegen;
mod interpreter;
mod lexer;
mod parser;
mod tokens;

fn main() {
    let input = r#"
(@fn add_times_2[]
    (* (+ 1.0 2.0) 3.0))

"#;

    let tokens = Lexer::new(input.to_string()).lex().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast).unwrap();

    println!("{}", compile_to_llvm_ir(&interpreter).unwrap());
}
