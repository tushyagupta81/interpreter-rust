use crate::{
    expr::{Expr, LiteralValue},
    stmt::Stmt,
};
use std::error::Error;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    pub fn interpret_expr(&mut self, expr: Expr) -> Result<LiteralValue, Box<dyn Error>> {
        expr.evaluvate()
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), Box<dyn Error>> {
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expression } => {
                    expression.evaluvate()?;
                }
                Stmt::Print { expression } => {
                    let val = expression.evaluvate()?;
                    println!("{}", val.to_string());
                }
            };
        }
        Ok(())
    }
}
