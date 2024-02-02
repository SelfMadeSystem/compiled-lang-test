use crate::ast::BinaryOp;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn kind_string(&self) -> String {
        match &self.kind {
            TokenKind::Number(s) => format!("Number({})", s),
            TokenKind::Input => "Input".to_string(),
            TokenKind::Plus => "+".to_string(),
            TokenKind::Minus => "-".to_string(),
            TokenKind::Star => "*".to_string(),
            TokenKind::Slash => "/".to_string(),
            TokenKind::LParen => "(".to_string(),
            TokenKind::RParen => ")".to_string(),
            TokenKind::EOF => "EOF".to_string(),
        }
    }    
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Number(String),
    Input,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    EOF,
}

impl TokenKind {
    pub fn number(&self) -> Option<f64> {
        match self {
            TokenKind::Number(s) => s.parse().ok(),
            _ => None,
        }
    }


    pub fn binary_op(&self) -> Option<BinaryOp> {
        match self {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            _ => None,
        }
    }
}
