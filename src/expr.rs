use super::scanner::Token;
use crate::{environments::Environment, interpreter::Interpreter, scanner, stmt::Stmt, TokenType};
use std::hash::Hasher;
use std::{cell::RefCell, error::Error, hash::Hash, rc::Rc};

// unwraping helper function
fn unwrap_as_f64(literal: Option<scanner::LiteralValue>) -> f64 {
    match literal {
        Some(scanner::LiteralValue::FloatValue(x)) => x,
        _ => panic!("Couldnt unwrap as f64"),
    }
}

// unwraping helper function
fn unwrap_as_string(literal: Option<scanner::LiteralValue>) -> String {
    match literal {
        Some(scanner::LiteralValue::StringValue(s)) => s.clone(),
        _ => panic!("Couldnt unwrap to string"),
    }
}

#[derive(Clone)]
pub enum LiteralValue {
    Number(f64),
    StringValue(String),
    True,
    False,
    Nil,
    Callable {
        name: String,
        arity: usize,
        #[allow(clippy::type_complexity)]
        fun: Rc<dyn Fn(&Vec<LiteralValue>) -> LiteralValue>,
    },
}

impl std::fmt::Debug for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for LiteralValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LiteralValue::Number(x), LiteralValue::Number(y)) => x == y,
            (LiteralValue::StringValue(x), LiteralValue::StringValue(y)) => x == y,
            (LiteralValue::False, LiteralValue::False) => true,
            (LiteralValue::True, LiteralValue::True) => true,
            (LiteralValue::False, LiteralValue::True) => false,
            (LiteralValue::True, LiteralValue::False) => false,
            (LiteralValue::Nil, LiteralValue::Nil) => true,
            (
                LiteralValue::Callable {
                    name,
                    arity,
                    fun: _,
                },
                LiteralValue::Callable {
                    name: name2,
                    arity: arity2,
                    fun: _,
                },
            ) => name == name2 && arity == arity2,
            _ => {
                panic!("Error in PartialEq of LiteralValue")
            }
        }
    }
}

#[allow(clippy::inherent_to_string)]
impl LiteralValue {
    pub fn to_string(&self) -> String {
        match self {
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::StringValue(s) => format!("\"{}\"", s),
            LiteralValue::True => "true".to_string(),
            LiteralValue::False => "false".to_string(),
            LiteralValue::Nil => "nil".to_string(),
            LiteralValue::Callable {
                name,
                arity,
                fun: _,
            } => format!("<fn {}>/{}", name, arity),
        }
    }

    pub fn to_type(&self) -> &str {
        match self {
            LiteralValue::Number(_) => "Number",
            LiteralValue::StringValue(_) => "String",
            LiteralValue::True | LiteralValue::False => "Boolean",
            LiteralValue::Nil => "Nil",
            LiteralValue::Callable {
                name: _,
                arity: _,
                fun: _,
            } => "Callable",
        }
    }

    // Create a TokenType from a given Token
    pub fn from_token(token: &Token) -> Self {
        match token.token_type {
            TokenType::Number => Self::Number(unwrap_as_f64(token.literal.clone())),
            TokenType::String_ => Self::StringValue(unwrap_as_string(token.literal.clone())),
            TokenType::True => Self::True,
            TokenType::False => Self::False,
            TokenType::Nil => Self::Nil,
            _ => panic!("Cannot create literal from {:?}", token),
        }
    }

    // Check is a given TokenType is False
    pub fn is_falsy(&self) -> LiteralValue {
        match self {
            LiteralValue::Number(e) => {
                if *e == 0. {
                    LiteralValue::True
                } else {
                    LiteralValue::False
                }
            }
            LiteralValue::StringValue(s) => {
                if s.is_empty() {
                    LiteralValue::True
                } else {
                    LiteralValue::False
                }
            }
            LiteralValue::False => LiteralValue::True,
            LiteralValue::True => LiteralValue::False,
            LiteralValue::Nil => LiteralValue::True,
            LiteralValue::Callable {
                name: _,
                arity: _,
                fun: _,
            } => {
                panic!("Cannot use callable as truthy value")
            }
        }
    }

    // Check is a given TokenType is True
    pub fn is_truthy(&self) -> LiteralValue {
        match self {
            LiteralValue::Number(e) => {
                if *e == 0. {
                    LiteralValue::False
                } else {
                    LiteralValue::True
                }
            }
            LiteralValue::StringValue(s) => {
                if s.is_empty() {
                    LiteralValue::False
                } else {
                    LiteralValue::True
                }
            }
            LiteralValue::True => LiteralValue::True,
            LiteralValue::False => LiteralValue::False,
            LiteralValue::Nil => LiteralValue::False,
            LiteralValue::Callable {
                name: _,
                arity: _,
                fun: _,
            } => {
                panic!("Cannot use callable as truthy value")
            }
        }
    }

    // Convert rust bool into LiteralValue bool
    pub fn from_bool(e: bool) -> Self {
        if e {
            LiteralValue::True
        } else {
            LiteralValue::False
        }
    }
}

#[derive(Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        literal: LiteralValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    #[allow(dead_code)]
    Call {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
    #[allow(clippy::vec_box)]
    AnonFunc {
        paren: Token,
        args: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state);
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        let ptr1 = std::ptr::addr_of!(self);
        let ptr2 = std::ptr::addr_of!(other);
        ptr1 == ptr2
    }
}

impl Eq for Expr {}

#[allow(clippy::inherent_to_string)]
impl Expr {
    pub fn to_string(&self) -> String {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left_str = (*left).to_string();
                let op = operator.lexeme.clone();
                let right_str = (*right).to_string();
                format!("({} {} {})", op, left_str, right_str)
            }
            Expr::Grouping { expression } => {
                format!("(group {})", (*expression).to_string())
            }
            Expr::Literal { literal } => literal.to_string(),
            Expr::Unary { operator, right } => {
                let op_str = operator.lexeme.clone();
                let right_str = (*right).to_string();
                format!("({} {})", op_str, right_str)
            }
            Expr::Variable { name } => {
                format!("(var {:?})", name)
            }
            Expr::Assign { name, value } => {
                format!("(assign {:?} {:?})", name, value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                format!(
                    "({} {} {})",
                    operator.token_type,
                    left.to_string(),
                    right.to_string()
                )
            }
            Expr::Call {
                callee,
                paren: _,
                args,
            } => {
                format!(
                    "<fn {}> {:?}",
                    callee.to_string(),
                    args //args.iter().map(|arg| arg.to_string()).collect::<String>()
                )
            }
            Expr::AnonFunc {
                args,
                body: _,
                paren: _,
            } => {
                format!("anon/{}", args.len())
            }
        }
    }

    // Evaluvate a Expression and return a LiteralValue
    pub fn evaluvate(
        &self,
        env: Rc<RefCell<Environment>>,
        distance: Option<usize>,
    ) -> Result<LiteralValue, Box<dyn Error>> {
        // Result is stored in res and returned as Ok(res) at end
        let res = match self {
            Expr::AnonFunc { paren, args, body } => {
                // Clone all params to prevent lifetime issues
                let arguments: Vec<Token> = args.iter().map(|t| (*t).clone()).collect();
                let body: Vec<Box<Stmt>> = body.iter().map(|b| (*b).clone()).collect();
                let paren_line = paren.line_number;

                let func_impl = move |args: &Vec<LiteralValue>| {
                    // Get the new Interpreter
                    let mut anon_env = Interpreter::for_anon(env.clone());
                    // Define all the parameters in the new Interpreter
                    for (i, arg) in args.iter().enumerate() {
                        anon_env
                            .environments
                            .borrow_mut()
                            .define(arguments[i].lexeme.clone(), arg.clone(),Some(0));
                    }
                    // Resolve the n-1 line in the body
                    #[allow(clippy::all)]
                    for i in 0..(body.len()) {
                        anon_env
                            .interpret(vec![body[i].as_ref()])
                            .unwrap_or_else(|_| {
                                panic!(
                                    "Evaluvation failed inside anon_func at line {}",
                                    paren_line.clone()
                                )
                            });
                        if let Some(val) = anon_env.specials.borrow_mut().get("return") {
                            return val.clone();
                        }
                    }
                    LiteralValue::Nil
                };

                LiteralValue::Callable {
                    name: "anon_function".to_string(),
                    arity: args.len(),
                    fun: Rc::from(func_impl),
                }
            }
            // If its a Variable Expression we try to get it and return its value
            Expr::Variable { name } => match env.borrow().get(&name.lexeme, distance) {
                Some(val) => val.clone(),
                None => return Err(format!("Variable '{}' is not defined", name.lexeme).into()),
            },
            // Function invokation here
            Expr::Call {
                callee,
                paren: _,
                args,
            } => {
                // First evaluvate the callee to get the invoking function defination
                let callable = callee.evaluvate(env.clone(), distance)?;
                match callable {
                    // Check if function defination matchs its invokation
                    LiteralValue::Callable { name, arity, fun } => {
                        // Check ig number of arguments are correct
                        if args.len() != arity {
                            return Err(format!(
                                "Callable '{}' expexted {} arguments and got {} arguments",
                                name,
                                arity,
                                args.len()
                            )
                            .into());
                        }
                        // Eval the args to literalvalue
                        let mut args_val = vec![];
                        for arg in args {
                            args_val.push(arg.evaluvate(env.clone(), distance)?)
                        }
                        // Call the fun with the args
                        fun(&args_val)
                    }
                    // If we dont get a callable type return error
                    e => return Err(format!("{} is not callable", e.to_type()).into()),
                }
            }
            // Assign a new value to a variable
            Expr::Assign { name, value } => {
                let new_value = (*value).evaluvate(env.clone(), distance)?;
                let assign_success =
                    env.borrow_mut()
                        .assign(&name.lexeme, new_value.clone(), distance);

                // If assignment is success return the value
                if assign_success {
                    return Ok(new_value);
                } else {
                    return Err(format!("Variable {} has not been declared", name.lexeme).into());
                }
            }
            // Logical OR and AND
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                // Get the lhs eq
                let lhs_expr = left.evaluvate(env.clone(), distance)?;

                if operator.token_type == TokenType::Or {
                    // If the operator is or and the LHS is true return it and dont compute RHS
                    if lhs_expr.is_truthy() == LiteralValue::True {
                        return Ok(lhs_expr);
                    }
                // If operator is AND and LHS is false, Return LHS
                } else if lhs_expr.is_falsy() == LiteralValue::True {
                    return Ok(lhs_expr);
                }
                // Otherwise return RHS
                let rhs_expr = right.evaluvate(env.clone(), distance)?;
                return Ok(rhs_expr);
            }
            Expr::Literal { literal } => literal.clone(),
            Expr::Grouping { expression } => expression.evaluvate(env, distance)?,
            Expr::Unary { operator, right } => {
                // Get the RHS
                let right = &right.evaluvate(env, distance)?;
                // Match the operation with the evaluvated expression
                match (right, &operator.token_type) {
                    (LiteralValue::Number(n), TokenType::Minus) => LiteralValue::Number(-n),
                    (any, TokenType::Bang) => any.is_falsy(),
                    _ => {
                        return Err(format!(
                            "{:?} Not not a valid Unary operator on {}",
                            &operator.token_type,
                            right.to_type()
                        )
                        .into())
                    }
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = &left.evaluvate(env.clone(), distance)?;
                let right = &right.evaluvate(env.clone(), distance)?;
                // Long match list of all possible(yet) binary operations
                match (left, right, &operator.token_type) {
                    (LiteralValue::Number(a), LiteralValue::Number(b), TokenType::Greater) => {
                        LiteralValue::from_bool(a > b)
                    }
                    (LiteralValue::Number(a), LiteralValue::Number(b), TokenType::GreaterEqual) => {
                        LiteralValue::from_bool(a >= b)
                    }
                    (LiteralValue::Number(a), LiteralValue::Number(b), TokenType::Less) => {
                        LiteralValue::from_bool(a < b)
                    }
                    (LiteralValue::Number(a), LiteralValue::Number(b), TokenType::LessEqual) => {
                        LiteralValue::from_bool(a <= b)
                    }
                    (
                        LiteralValue::StringValue(a),
                        LiteralValue::StringValue(b),
                        TokenType::Greater,
                    ) => LiteralValue::from_bool(a > b),
                    (
                        LiteralValue::StringValue(a),
                        LiteralValue::StringValue(b),
                        TokenType::GreaterEqual,
                    ) => LiteralValue::from_bool(a >= b),
                    (
                        LiteralValue::StringValue(a),
                        LiteralValue::StringValue(b),
                        TokenType::Less,
                    ) => LiteralValue::from_bool(a < b),
                    (
                        LiteralValue::StringValue(a),
                        LiteralValue::StringValue(b),
                        TokenType::LessEqual,
                    ) => LiteralValue::from_bool(a <= b),

                    (LiteralValue::Number(a), LiteralValue::Number(b), TokenType::Star) => {
                        LiteralValue::Number(a * b)
                    }
                    (LiteralValue::Number(a), LiteralValue::Number(b), TokenType::Slash) => {
                        LiteralValue::Number(a / b)
                    }
                    (LiteralValue::Number(a), LiteralValue::Number(b), TokenType::Minus) => {
                        LiteralValue::Number(a - b)
                    }

                    (LiteralValue::Number(a), LiteralValue::Number(b), TokenType::Plus) => {
                        LiteralValue::Number(a + b)
                    }
                    (
                        LiteralValue::StringValue(a),
                        LiteralValue::StringValue(b),
                        TokenType::Plus,
                    ) => LiteralValue::StringValue(format!("{}{}", a, b)),

                    (left, right, TokenType::EqualEqual) => LiteralValue::from_bool(left == right),
                    (left, right, TokenType::BangEqual) => LiteralValue::from_bool(left != right),
                    _ => {
                        return Err(format!(
                            "{} Not implemented on '{}' and '{}'",
                            &operator.token_type,
                            left.to_type(),
                            right.to_type()
                        )
                        .into())
                    }
                }
            }
        };
        Ok(res)
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        println!("{}", self.to_string());
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, usize};

    use super::*;
    use crate::scanner::TokenType;

    #[test]
    fn pretty_print_ast() {
        let minus_token = Token {
            token_type: TokenType::Minus,
            lexeme: "-".to_string(),
            literal: None,
            line_number: 1 as usize,
        };

        let onetwothree = Box::new(Expr::Literal {
            literal: LiteralValue::Number(123.0),
        });
        let multi = Token {
            token_type: TokenType::Star,
            lexeme: "*".to_string(),
            literal: None,
            line_number: 1 as usize,
        };
        let group = Box::new(Expr::Grouping {
            expression: Box::new(Expr::Literal {
                literal: LiteralValue::Number(45.67),
            }),
        });

        let ast = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: minus_token,
                right: onetwothree,
            }),
            operator: multi,
            right: group,
        };

        ast.print();
        assert_eq!(ast.to_string(), "(* (- 123) (group 45.67))".to_string());
    }

    #[test]
    fn expr_traits() {
        let mut hm = HashMap::new();

        let minus_token = Token {
            token_type: TokenType::Minus,
            lexeme: "-".to_string(),
            literal: None,
            line_number: 1 as usize,
        };

        let onetwothree = Box::new(Expr::Literal {
            literal: LiteralValue::Number(123.0),
        });
        let multi = Token {
            token_type: TokenType::Star,
            lexeme: "*".to_string(),
            literal: None,
            line_number: 1 as usize,
        };
        let group = Box::new(Expr::Grouping {
            expression: Box::new(Expr::Literal {
                literal: LiteralValue::Number(45.67),
            }),
        });

        let ast = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: minus_token,
                right: onetwothree,
            }),
            operator: multi,
            right: group,
        };

        let ast = std::rc::Rc::new(ast);
        hm.insert(ast.clone(), 2);
        match hm.get(&ast) {
            Some(_) => (),
            None => panic!("Should be able to get the value in trait expr"),
        }

        let minus_token = Token {
            token_type: TokenType::Minus,
            lexeme: "-".to_string(),
            literal: None,
            line_number: 1 as usize,
        };

        let onetwothree = Box::new(Expr::Literal {
            literal: LiteralValue::Number(123.0),
        });
        let multi = Token {
            token_type: TokenType::Star,
            lexeme: "*".to_string(),
            literal: None,
            line_number: 1 as usize,
        };
        let group = Box::new(Expr::Grouping {
            expression: Box::new(Expr::Literal {
                literal: LiteralValue::Number(45.67),
            }),
        });

        let ast = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: minus_token,
                right: onetwothree,
            }),
            operator: multi,
            right: group,
        };

        let ast = std::rc::Rc::new(ast);
        match hm.get(&ast) {
            None => (),
            Some(_) => panic!("Should get None in expr traits"),
        }
    }
}
