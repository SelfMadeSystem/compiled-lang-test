use anyhow::Result;
use codegen::run_ast;
use lexer::Lexer;
use parser::Parser;

mod ast;
mod codegen;
mod lexer;
mod parser;
mod tokens;

fn main() -> Result<()> {
    let input = "1 + 2 * - ( 3 - 4 / ( - 5 ) )";

    let tokens = Lexer::new(input.to_string()).lex().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    run_ast(ast)
}
