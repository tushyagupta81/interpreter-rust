use crate::{
    environments::Environments,
    stmt::Stmt,
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

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), Box<dyn Error>> {
        for stmt in stmts {
            match stmt {
                Stmt::Expression { expression } => {
                    expression.evaluvate(&self.environments)?;
                }
                Stmt::Print { expression } => {
                    let val = expression.evaluvate(&self.environments)?;
                    println!("{}", val.to_string());
                }
                Stmt::Var { name, initializer } => {
                    let val = initializer.evaluvate(&self.environments)?;

                    self.environments.define(name.lexeme, val);
                }
            };
        }
        Ok(())
    }
}
