use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

// mod codegen;
mod interpreter;
mod lexer;
mod parser;
mod tokens;

fn main() {
    let input = r#"
(@fn * [a, b])
(@fn + [a, b])

(@fn add_times_2[a, b]
    (* (+ a b) 3.0))
"#;

    let tokens = Lexer::new(input.to_string()).lex().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast).unwrap();

    println!("{:#?}", interpreter);
}
