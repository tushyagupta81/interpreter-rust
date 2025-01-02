use crate::{environments::Environments, expr::LiteralValue, stmt::Stmt};
use std::error::Error;
use std::rc::Rc;

pub struct Interpreter {
    environments: Rc<Environments>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environments: Rc::new(Environments::new()),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<Option<LiteralValue>, Box<dyn Error>> {
        for stmt in stmts {
            match stmt {
                Stmt::WhileLoop { cond, body } => {
                    todo!()
                }
                Stmt::Expression { expression } => {
                    expression.evaluvate(
                        Rc::get_mut(&mut self.environments)
                            .expect("Failed to get mut reference to env"),
                    )?;
                }
                Stmt::Print { expression } => {
                    let val = expression.evaluvate(
                        Rc::get_mut(&mut self.environments)
                            .expect("Failed to get mut reference to env"),
                    )?;

                    println!("{}", val.to_string());
                }
                Stmt::Var { name, initializer } => {
                    let val = initializer.evaluvate(
                        Rc::get_mut(&mut self.environments)
                            .expect("Failed to get mut reference to env"),
                    )?;

                    Rc::get_mut(&mut self.environments)
                        .expect("Failed to get mut reference to env")
                        .define(name.lexeme, val);
                }
                Stmt::Block { stmts } => {
                    let mut new_env = Environments::new();
                    new_env.enclosing = Some(self.environments.clone());

                    let old_env = self.environments.clone();
                    self.environments = Rc::new(new_env);
                    let block_res = self.interpret(stmts);
                    self.environments = old_env;

                    block_res?;
                }
                Stmt::IfElse {
                    predicate,
                    then_branch,
                    else_branch,
                } => {
                    let truth_val = predicate.evaluvate(
                        Rc::get_mut(&mut self.environments)
                            .expect("Failed to get mut reference to env"),
                    )?;
                    if truth_val.is_truthy() == LiteralValue::True {
                        self.interpret(vec![*then_branch])?;
                    } else if let Some(stmt) = else_branch {
                        self.interpret(vec![*stmt])?;
                    }
                }
            };
        }
        Ok(None)
    }
}
