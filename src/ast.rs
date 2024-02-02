#[derive(Debug, PartialEq)]
pub struct Ast {
    pub kind: AstKind,
    pub line: usize,
    pub column: usize,
}

impl Ast {
    pub fn to_str_expr(&self) -> String {
        match &self.kind {
            AstKind::Number(n) => n.to_string(),
            AstKind::Input => "i".to_string(),
            AstKind::BinaryOp { op, lhs, rhs } => {
                format!(
                    "({} {} {})",
                    lhs.to_str_expr(),
                    match op {
                        BinaryOp::Add => "+",
                        BinaryOp::Sub => "-",
                        BinaryOp::Mul => "*",
                        BinaryOp::Div => "/",
                    },
                    rhs.to_str_expr()
                )
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AstKind {
    Number(f64),
    Input,
    BinaryOp {
        op: BinaryOp,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl BinaryOp {
    pub fn precedence(&self) -> usize {
        match self {
            BinaryOp::Add | BinaryOp::Sub => 1,
            BinaryOp::Mul | BinaryOp::Div => 2,
        }
    }
}
