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
    IfElse {
        predicate: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    WhileLoop {
        cond: Expr,
        body: Box<Stmt>,
    },
}

#[allow(clippy::inherent_to_string,dead_code,unused_variables)]
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
            Stmt::IfElse {
                predicate,
                then_branch,
                else_branch,
            } => {
                todo!()
            }
            Stmt::WhileLoop { cond, body } => {
                todo!()
            }
        }
    }
}
