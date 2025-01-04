use super::scanner::Token;
use crate::{environments::Environments, scanner, TokenType};
use std::{cell::RefCell, error::Error, rc::Rc};

fn unwrap_as_f64(literal: Option<scanner::LiteralValue>) -> f64 {
    match literal {
        Some(scanner::LiteralValue::FloatValue(x)) => x,
        _ => panic!("Couldnt unwrap as f64"),
    }
}

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
        fun: Rc<dyn Fn(Rc<RefCell<Environments>>, &Vec<LiteralValue>) -> LiteralValue>,
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
            _ => todo!(),
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
            } => format!("{}/{}", name, arity),
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
    Call {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Expr>,
    },
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

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
                    "{} {:?}",
                    callee.to_string(),
                    args //args.iter().map(|arg| arg.to_string()).collect::<String>()
                )
            }
        }
    }

    pub fn evaluvate(
        &self,
        env: Rc<RefCell<Environments>>,
    ) -> Result<LiteralValue, Box<dyn Error>> {
        let res = match self {
            Expr::Variable { name } => match env.borrow().get(&name.lexeme) {
                Some(val) => val.clone(),
                None => return Err(format!("Variable '{}' is not defined", name.lexeme).into()),
            },
            Expr::Call {
                callee,
                paren: _,
                args,
            } => {
                let callable = callee.evaluvate(env.clone())?;
                match callable {
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
                            args_val.push(arg.evaluvate(env.clone())?)
                        }
                        // Call the fun with the args
                        fun(env.clone(),&args_val)
                    }
                    e => return Err(format!("{} is not callable", e.to_type()).into()),
                }
            }
            Expr::Assign { name, value } => {
                let new_value = (*value).evaluvate(env.clone())?;
                let assign_success = env.borrow_mut().assign(&name.lexeme, new_value.clone());

                if assign_success {
                    return Ok(new_value);
                } else {
                    return Err(format!("Variable {} has not been declared", name.lexeme).into());
                }
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let lhs_expr = left.evaluvate(env.clone())?;

                if operator.token_type == TokenType::Or {
                    if lhs_expr.is_truthy() == LiteralValue::True {
                        return Ok(lhs_expr);
                    }
                } else if lhs_expr.is_falsy() == LiteralValue::True {
                    return Ok(lhs_expr);
                }
                let rhs_expr = right.evaluvate(env.clone())?;
                return Ok(rhs_expr);
            }
            Expr::Literal { literal } => literal.clone(),
            Expr::Grouping { expression } => expression.evaluvate(env)?,
            Expr::Unary { operator, right } => {
                let right = &right.evaluvate(env)?;
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
                let left = &left.evaluvate(env.clone())?;
                let right = &right.evaluvate(env.clone())?;
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
    use std::usize;

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
}
