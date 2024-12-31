use crate::{
    environments::Environments,
    stmt::Stmt, expr::LiteralValue,
};
use std::error::Error;

pub struct Interpreter {
    environments: Environments,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environments: Environments::new(),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<Option<LiteralValue>, Box<dyn Error>> {
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expression } => {
                    expression.evaluvate(&mut self.environments)?;
                }
                Stmt::Print { expression } => {
                    let val = expression.evaluvate(&mut self.environments)?;
                    println!("{}", val.to_string());
                }
                Stmt::Var { name, initializer } => {
                    let val = initializer.evaluvate(&mut self.environments)?;

                    self.environments.define(name.lexeme, val);
                }
            };
        }
        Ok(None)
    }
}
