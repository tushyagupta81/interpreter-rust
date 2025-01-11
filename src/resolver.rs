use std::{collections::HashMap, error::Error};

use crate::{expr::Expr, interpreter::Interpreter, stmt::Stmt, Token};

#[allow(dead_code)]
pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

#[allow(dead_code)]
impl Resolver {
    pub fn new() -> Self {
        Resolver {
            interpreter: Interpreter::new(),
            scopes: vec![],
        }
    }

    pub fn resolve(&mut self, stmt: &Stmt) -> Result<(), Box<dyn Error>> {
        match stmt {
            Stmt::Block { stmts: _ } => {
                self.resolve_block(stmt)?;
            }
            Stmt::Var {
                name: _,
                initializer: _,
            } => {
                self.resolve_var(stmt)?;
            }
            Stmt::Function {
                name: _,
                params: _,
                body: _,
            } => {
                self.resolve_function(stmt)?;
            }
            Stmt::Expression { expression } => {
                self.resolve_expr(expression)?;
            }
            Stmt::IfElse {
                predicate: _,
                then_branch: _,
                else_branch: _,
            } => {
                self.resolve_if_else(stmt)?;
            }
            Stmt::Print { expression } => {
                self.resolve_expr(expression)?;
            }
            Stmt::Return { keyword: _, value } => {
                if let Some(val) = value {
                    self.resolve_expr(val)?;
                }
            }
            Stmt::WhileLoop { cond, body } => {
                self.resolve_expr(cond)?;
                self.resolve(body)?;
            }
        }
        Ok(())
    }

    fn resolve_if_else(&mut self, stmt: &Stmt) -> Result<(), Box<dyn Error>> {
        match stmt {
            Stmt::IfElse {
                predicate,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(predicate)?;
                self.resolve(then_branch)?;
                if let Some(els) = else_branch {
                    self.resolve(els.as_ref())?;
                }
            }
            _ => panic!("Wrong type in resolve if else"),
        }
        Ok(())
    }

    fn resolve_function(&mut self, stmt: &Stmt) -> Result<(), Box<dyn Error>> {
        match stmt {
            Stmt::Function { name, params, body } => {
                self.declare(name)?;
                self.define(name)?;
                self.resolve_function_helper(params, body)?;
            }
            _ => panic!("Wrong type in resolve function"),
        }
        Ok(())
    }

    #[allow(clippy::vec_box)]
    fn resolve_function_helper(
        &mut self,
        params: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
    ) -> Result<(), Box<dyn Error>> {
        self.begin_scope()?;
        for param in params {
            self.declare(param)?;
            self.define(param)?;
        }
        self.resolve_many(body)?;
        self.end_scope()?;
        Ok(())
    }

    #[allow(clippy::vec_box)]
    fn resolve_many(&mut self, stmts: &Vec<Box<Stmt>>) -> Result<(), Box<dyn Error>> {
        for stmt in stmts {
            self.resolve(stmt.as_ref())?;
        }
        Ok(())
    }

    fn resolve_var(&mut self, stmt: &Stmt) -> Result<(), Box<dyn Error>> {
        match stmt {
            Stmt::Var { name, initializer } => {
                self.declare(name)?;
                self.resolve_expr(initializer)?;
                self.define(name)?;
                Ok(())
            }
            _ => panic!("Wrong tpye in resolve var stmt"),
        }
    }

    fn declare(&mut self, name: &Token) -> Result<(), Box<dyn Error>> {
        if self.scopes.is_empty() {
            return Ok(());
        }
        self.scopes
            .last_mut()
            .expect("No scope found while declare")
            .insert(name.lexeme.clone(), false);
        Ok(())
    }

    fn define(&mut self, name: &Token) -> Result<(), Box<dyn Error>> {
        if self.scopes.is_empty() {
            return Ok(());
        }
        self.scopes
            .last_mut()
            .expect("No scope found while define")
            .insert(name.lexeme.clone(), true);
        Ok(())
    }

    #[allow(clippy::vec_box)]
    fn resolve_block(&mut self, stmt: &Stmt) -> Result<(), Box<dyn Error>> {
        match stmt {
            Stmt::Block { stmts } => {
                self.begin_scope()?;
                self.resolve_many(stmts)?;
                self.end_scope()?;
            }
            _ => panic!("Wrong tpye in resolve block"),
        }
        Ok(())
    }

    fn begin_scope(&mut self) -> Result<(), Box<dyn Error>> {
        self.scopes.push(HashMap::new());
        Ok(())
    }

    fn end_scope(&mut self) -> Result<(), Box<dyn Error>> {
        self.scopes.pop().expect("Stack underflow during scope");
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), Box<dyn Error>> {
        match expr {
            Expr::Variable { name: _ } => {
                self.resolve_expr_var(expr)?;
            }
            Expr::Assign { name: _, value: _ } => {
                self.resolve_expr_assign(expr)?;
            }
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            Expr::Call {
                callee,
                paren: _,
                args,
            } => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            }
            Expr::Grouping { expression } => {
                self.resolve_expr(expression)?;
            }
            Expr::Literal { literal: _ } => {}
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            Expr::Unary { operator: _, right } => {
                self.resolve_expr(right)?;
            }
            Expr::AnonFunc {
                paren: _,
                args,
                body,
            } => {
                self.resolve_function_helper(args, body)?;
            }
        }
        Ok(())
    }

    fn resolve_expr_assign(&mut self, expr: &Expr) -> Result<(), Box<dyn Error>> {
        match expr {
            Expr::Assign { name, value } => {
                self.resolve_expr(value.as_ref())?;
                self.resolve_local(expr, name)?;
            }
            _ => panic!("Wrong type in resolve assign"),
        }
        Ok(())
    }

    fn resolve_expr_var(&mut self, expr: &Expr) -> Result<(), Box<dyn Error>> {
        match expr {
            Expr::Variable { name } => {
                if !self.scopes.is_empty()
                    && !(*self
                        .scopes
                        .last()
                        .expect("No scopes during var expr")
                        .get(&name.lexeme)
                        .unwrap())
                {
                    return Err("Cannot read local variable in its own initialization".into());
                }
                self.resolve_local(expr, name)?;
            }
            _ => panic!("Wrong type in resolve var"),
        }
        Ok(())
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) -> Result<(), Box<dyn Error>> {
        let size = self.scopes.len();
        for i in (size - 1)..0 {
            if self.scopes[i].contains_key(&name.lexeme) {
                self.interpreter
                    .resolve(expr, (size as i32) - 1 - (i as i32))?;
                return Ok(());
            }
        }
        Ok(())
    }
}
