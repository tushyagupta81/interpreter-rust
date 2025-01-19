use crate::expr::Expr;
use crate::Token;
use crate::{environments::Environment, expr::LiteralValue, stmt::Stmt};
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

// Main heart of the operation
pub struct Interpreter {
    pub specials: Rc<RefCell<Environment>>,
    pub environments: Rc<RefCell<Environment>>,
    // globals: HashMap<String, LiteralValue>,
    pub locals: Rc<RefCell<HashMap<usize, usize>>>,
}

impl Interpreter {
    pub fn new() -> Self {
        // Define the STD lib functions on startup
        Self {
            specials: Rc::new(RefCell::new(Environment::new())),
            environments: Rc::new(RefCell::new(Environment::new())),
            // globals: Interpreter::get_globals(),
            locals: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    // Return a new Interpreter with a enclosing parent of another Interpreter
    fn for_closure(parent: Rc<RefCell<Environment>>) -> Self {
        let environments = Rc::new(RefCell::new(Environment::new()));
        environments.borrow_mut().enclosing = Some(parent);
        Interpreter {
            specials: Rc::new(RefCell::new(Environment::new())),
            environments,
            // globals: Interpreter::get_globals(),
            locals: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn for_anon(parent: Rc<RefCell<Environment>>) -> Self {
        let environments = Rc::new(RefCell::new(Environment::new()));
        environments.borrow_mut().enclosing = Some(parent);
        Interpreter {
            specials: Rc::new(RefCell::new(Environment::new())),
            environments,
            // globals: Interpreter::get_globals(),
            locals: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    #[allow(clippy::let_and_return)]
    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<Option<LiteralValue>, Box<dyn Error>> {
        for stmt in stmts {
            match stmt {
                Stmt::Return { keyword: _, value } => {
                    let value = match value {
                        Some(expr) => expr.evaluvate(self.environments.clone())?,
                        None => LiteralValue::Nil,
                    };

                    self.specials
                        .borrow_mut()
                        .define_top_level("return".to_string(), value);
                }
                // Mother of hell ah function
                Stmt::Function { name, params, body } => {
                    // Get the arity
                    let arity = params.len();

                    // Clone all params to prevent lifetime issues
                    let params: Vec<Token> = params.iter().map(|t| (*t).clone()).collect();
                    let body: Vec<Box<Stmt>> = body.iter().map(|b| (*b).clone()).collect();
                    let name_clone = name.lexeme.clone();

                    let parent_env = self.environments.clone();
                    // Make a function implementaion
                    let func_impl = move |args: &Vec<LiteralValue>| {
                        // Get the new Interpreter
                        let mut closure_interpreter = Interpreter::for_closure(parent_env.clone());
                        // Define all the parameters in the new Interpreter
                        for (i, arg) in args.iter().enumerate() {
                            closure_interpreter
                                .environments
                                .borrow_mut()
                                .define(params[i].lexeme.clone(), arg.clone());
                        }
                        // Resolve the n-1 line in the body
                        #[allow(clippy::all)]
                        for i in 0..(body.len()) {
                            closure_interpreter
                                .interpret(vec![body[i].as_ref()])
                                .unwrap_or_else(|_| {
                                    panic!("Evaluvation failed inside {:?}", name_clone)
                                });
                            if let Some(val) = closure_interpreter.specials.borrow().get("return") {
                                return val;
                            }
                        }
                        LiteralValue::Nil
                    };
                    // Create a Callable
                    let callable = LiteralValue::Callable {
                        //name: name.lexeme.clone(),
                        name: name.to_string(),
                        arity,
                        fun: Rc::from(func_impl),
                    };

                    // Initialize the Callable in the Environment(parent Interpreter here)
                    self.environments
                        .borrow_mut()
                        .define(name.lexeme.clone(), callable);
                }
                // Keep executing a Block till the time the flag is true
                Stmt::WhileLoop { cond, body } => {
                    let mut flag = cond.evaluvate(self.environments.clone())?;
                    while flag.is_truthy() == LiteralValue::True {
                        self.interpret(vec![body.as_ref()])?;
                        flag = cond.evaluvate(self.environments.clone())?;
                    }
                }
                // Execute a expresssion regularly
                Stmt::Expression { expression } => {
                    let distance = self.get_distance(&expression);
                    expression.evaluvate(self.environments.clone())?;
                }
                // Evaluvate the value and then print it out
                Stmt::Print { expression } => {
                    let val = expression.evaluvate(self.environments.clone())?;

                    println!("{}", val.to_string());
                }
                // For a variable resolve its value and then define it in the Environment
                Stmt::Var { name, initializer } => {
                    let val = initializer.evaluvate(self.environments.clone())?;

                    self.environments
                        .borrow_mut()
                        .define(name.lexeme.clone(), val);
                }
                // Make a new Environment, make it the main Environment and make the enclsing the
                // orignal Environment to run the block
                // Restore the old Environment when finished with the block
                Stmt::Block { stmts } => {
                    let mut new_env = Environment::new();
                    new_env.enclosing = Some(self.environments.clone());

                    let old_env = self.environments.clone();
                    self.environments = Rc::new(RefCell::new(new_env));
                    let block_res =
                        self.interpret((*stmts).iter().map(|b| b.as_ref()).collect::<Vec<&Stmt>>());
                    self.environments = old_env;

                    block_res?;
                }
                // If the condition is true Execute the then_branch else do the else_branch
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

    pub fn resolve(&mut self, expr: &Expr, size: usize) -> Result<(), Box<dyn Error>> {
        let addr = std::ptr::addr_of!(expr) as usize;
        self.locals.borrow_mut().insert(addr, size);
        Ok(())
    }

    fn get_distance(&self, expr: &Expr) -> Option<usize> {
        let addr = std::ptr::addr_of!(expr) as usize;
        self.locals.borrow().get(&addr).copied()
    }
}
