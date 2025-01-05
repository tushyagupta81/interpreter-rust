use crate::expr::Expr;
use crate::scanner::Token;

#[derive(Debug,Clone)]
#[allow(clippy::vec_box)]
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
        stmts: Vec<Box<Stmt>>,
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
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Stmt>>
    },
    #[allow(dead_code)]
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
}

#[allow(clippy::inherent_to_string, dead_code)]
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
                predicate: _,
                then_branch: _,
                else_branch: _,
            } => {
                todo!()
            }
            Stmt::WhileLoop { cond: _, body: _ } => {
                todo!()
            }
            Stmt::Function { name:_, params:_, body:_ } => {
                todo!()
            }
            Stmt::Return {keyword:_, value:_ } => todo!()
        }
    }
}
