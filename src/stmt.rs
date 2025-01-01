use crate::expr::Expr;
use crate::scanner::Token;

#[derive(Debug)]
pub enum Stmt {
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Expr,
    },
    Block {
        stmts: Vec<Stmt>,
    },
    IfStmt {
        predicate: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
}

#[allow(clippy::inherent_to_string)]
#[allow(dead_code)]
impl Stmt {
    pub fn to_string(&self) -> String {
        match self {
            Stmt::Var {
                name,
                initializer: _,
            } => format!("(var {})", name.lexeme),
            Stmt::Print { expression } => format!("(print {})", expression.to_string()),
            Stmt::Expression { expression } => expression.to_string(),
            Stmt::Block { stmts } => stmts
                .iter()
                .map(|stmt| stmt.to_string())
                .collect::<String>(),
            Stmt::IfStmt { predicate, then_branch, else_branch } => todo!()
        }
    }
}
