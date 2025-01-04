use crate::Token;
use crate::{environments::Environments, expr::LiteralValue, stmt::Stmt};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

pub struct Interpreter {
    //globals: Environments,
    environments: Rc<RefCell<Environments>>,
}

fn clock_impl(_parent_env: Rc<RefCell<Environments>>, _args: &Vec<LiteralValue>) -> LiteralValue {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Could not get system time")
        .as_millis();

    LiteralValue::Number(now as f64 / 1000.0)
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environments::new();
        globals.define(
            "clock".to_string(),
            LiteralValue::Callable {
                name: "clock".to_string(),
                arity: 0,
                fun: Rc::from(clock_impl),
            },
        );
        Self {
            //globals,
            //environments: Rc::new(RefCell::new(Environments::new())),
            environments: Rc::new(RefCell::new(globals)),
        }
    }

    fn for_closure(parent: Rc<RefCell<Environments>>) -> Self {
        let environments = Rc::new(RefCell::new(Environments::new()));
        environments.borrow_mut().enclosing = Some(parent);
        Interpreter { environments }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<Option<LiteralValue>, Box<dyn Error>> {
        for stmt in stmts {
            match stmt {
                Stmt::Function { name, params, body } => {
                    let arity = params.len();

                    let params: Vec<Token> = params.iter().map(|t| (*t).clone()).collect();
                    let body: Vec<Box<Stmt>> = body.iter().map(|b| (*b).clone()).collect();
                    let name_clone = name.lexeme.clone();

                    let func_impl = move |parent_env, args: &Vec<LiteralValue>| {
                        let mut closure_interpreter = Interpreter::for_closure(parent_env);
                        for (i, arg) in args.iter().enumerate() {
                            closure_interpreter
                                .environments
                                .borrow_mut()
                                .define(params[i].lexeme.clone(), arg.clone());
                        }
                        for i in 0..(body.len() - 1) {
                            closure_interpreter
                                .interpret(vec![body[i].as_ref()])
                                .expect(
                                    format!("Evaluvation failed inside {:?}", name_clone)
                                        .as_str(),
                                );
                        }
                        let val;
                        match &*body[body.len() - 1] {
                            Stmt::Expression { expression } => {
                                val = expression
                                    .evaluvate(closure_interpreter.environments.clone())
                                    .expect(
                                        format!(
                                            "Evaluvation failed inside {:?} while getting value",
                                            name_clone
                                        )
                                        .as_str(),
                                    );
                            }
                            _ => todo!(),
                        };
                        val
                    };
                    let callable = LiteralValue::Callable {
                        name: name.to_string(),
                        arity,
                        fun: Rc::from(func_impl),
                    };

                    self.environments
                        .borrow_mut()
                        .define(name.lexeme.clone(), callable);
                }
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
