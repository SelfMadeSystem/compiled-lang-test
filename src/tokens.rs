use std::fmt::Display;

pub const DELIMITERS: &str = "(){}[],:;";

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token({}, {}:{})", self.kind, self.line, self.column)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),     // 'a'
    String(String), // "hello, world"
    Identifier(Identifier),
    Delimiter(char),
    EOF,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Int(n) => write!(f, "Int({})", n),
            TokenKind::Float(n) => write!(f, "Float({})", n),
            TokenKind::Bool(b) => write!(f, "Bool({})", b),
            TokenKind::Char(c) => write!(f, "Char({})", c),
            TokenKind::String(s) => write!(f, "String({})", s),
            TokenKind::Identifier(id) => write!(f, "Identifier({:?})", id),
            TokenKind::Delimiter(c) => write!(f, "Delimiter({})", c),
            TokenKind::EOF => write!(f, "EOF"),
        }
    }
}

impl TokenKind {
    pub fn is_literal(&self) -> bool {
        match self {
            TokenKind::Int(_)
            | TokenKind::Float(_)
            | TokenKind::Bool(_)
            | TokenKind::Char(_)
            | TokenKind::String(_)
            | TokenKind::Identifier(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct Identifier {
    pub name: String,
    pub kind: IdentifierKind,
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum IdentifierKind {
    Variable, // i
    Macro,    // @macro
    Type,     // $type
}
