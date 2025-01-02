use crate::{environments::Environments, expr::LiteralValue, stmt::Stmt};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

pub struct Interpreter {
    environments: Rc<RefCell<Environments>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environments: Rc::new(RefCell::new(Environments::new())),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<Option<LiteralValue>, Box<dyn Error>> {
        for stmt in stmts {
            match stmt {
                Stmt::WhileLoop { cond, body } => {
                    let mut flag = cond.evaluvate(self.environments.clone())?;
                    while flag.is_truthy() == LiteralValue::True {
                        self.interpret(vec![body.as_ref()])?;
                        flag = cond.evaluvate(self.environments.clone())?;
                    }
                }
                Stmt::Expression { expression } => {
                    expression.evaluvate(self.environments.clone())?;
                }
                Stmt::Print { expression } => {
                    let val = expression.evaluvate(self.environments.clone())?;

                    println!("{}", val.to_string());
                }
                Stmt::Var { name, initializer } => {
                    let val = initializer.evaluvate(self.environments.clone())?;

                    self.environments
                        .borrow_mut()
                        .define(name.lexeme.clone(), val);
                }
                Stmt::Block { stmts } => {
                    let mut new_env = Environments::new();
                    new_env.enclosing = Some(self.environments.clone());

                    let old_env = self.environments.clone();
                    self.environments = Rc::new(RefCell::new(new_env));
                    let block_res =
                        self.interpret((*stmts).iter().map(|b| b.as_ref()).collect::<Vec<&Stmt>>());
                    self.environments = old_env;

                    block_res?;
                }
                Stmt::IfElse {
                    predicate,
                    then_branch,
                    else_branch,
                } => {
                    let truth_val = predicate.evaluvate(self.environments.clone())?;
                    if truth_val.is_truthy() == LiteralValue::True {
                        self.interpret(vec![then_branch.as_ref()])?;
                    } else if let Some(stmt) = else_branch {
                        self.interpret(vec![stmt.as_ref()])?;
                    }
                }
            };
        }
        Ok(None)
    }
}
