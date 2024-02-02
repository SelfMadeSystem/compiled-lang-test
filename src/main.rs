use codegen::{compile_to_llvm_ir, run_ast};
use lexer::Lexer;
use parser::Parser;

use crate::codegen::compile_to_file;

mod ast;
mod codegen;
mod lexer;
mod parser;
mod tokens;

fn main() {
    let input = "i + 1 * (2 - 3 / i)";

    let tokens = Lexer::new(input.to_string()).lex().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let ir = compile_to_llvm_ir(&ast).expect("Unable to compile to LLVM IR");
    println!("{}", ir);

    compile_to_file(&ast, "sum").expect("Unable to compile to file");

    let result = run_ast(&ast).expect("Unable to run AST");
    println!("{} = {}", ast.to_str_expr(), result);
}
