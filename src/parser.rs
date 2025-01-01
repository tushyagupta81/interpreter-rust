use crate::expr::*;
use crate::scanner::Token;
use crate::scanner::TokenType::*;
use crate::stmt::Stmt;
use crate::TokenType;
use std::error::Error;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Box<dyn Error>> {
        let mut stmts = vec![];
        let mut errors = vec![];

        while !self.is_at_end() {
            let stmt = self.declaration();
            match stmt {
                Ok(s) => stmts.push(s),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }

        if errors.is_empty() {
            Ok(stmts)
        } else {
            let mut err = String::new();
            for error in errors {
                err.push_str(format!("{}{}", &error.to_string(), "\n").as_str());
            }
            Err(err.into())
        }
    }

    fn declaration(&mut self) -> Result<Stmt, Box<dyn Error>> {
        if self.match_token(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, Box<dyn Error>> {
        let token = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initializer = if self.match_token(Equal) {
            self.expression()?
        } else {
            Expr::Literal {
                literal: LiteralValue::Nil,
            }
        };

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        Ok(Stmt::Var {
            name: token,
            initializer,
        })
    }

    fn statement(&mut self) -> Result<Stmt, Box<dyn Error>> {
        if self.match_token(Print) {
            self.print_expression()
        } else if self.match_token(LeftBrace) {
            self.block()
        } else if self.match_token(If) {
            self.if_statement()
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, Box<dyn Error>> {
        self.consume(LeftParen, "Expected '(' after 'if'")?;
        let predicate = self.expression()?;
        self.consume(RightParen, "Expected ')' after if-predicate")?;
        let then_branch = Box::from(self.statement()?);
        let else_branch = if self.match_token(Else) {
            let stmt = self.statement()?;
            Some(Box::from(stmt))
        } else {
            None
        };

        Ok(Stmt::IfStmt {
            predicate,
            then_branch,
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Stmt, Box<dyn Error>> {
        let mut stmts = vec![];

        while !self.check(RightBrace) && !self.is_at_end() {
            let stmt = self.declaration()?;
            stmts.push(stmt);
        }

        self.consume(RightBrace, "Expect '}' after block.")?;

        Ok(Stmt::Block { stmts })
    }

    fn print_expression(&mut self) -> Result<Stmt, Box<dyn Error>> {
        let val = self.expression()?;
        self.consume(Semicolon, "Expected ';' after value")?;
        Ok(Stmt::Print { expression: val })
    }

    fn expression_statement(&mut self) -> Result<Stmt, Box<dyn Error>> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::Expression { expression: expr })
    }

    fn expression(&mut self) -> Result<Expr, Box<dyn Error>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Box<dyn Error>> {
        let lhs_expr = self.equality()?;

        if self.match_token(Equal) {
            let _eq = self.previous();
            let rhs_expr = self.assignment()?;
            match lhs_expr {
                Expr::Variable { name } => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::from(rhs_expr),
                    });
                }
                _ => {
                    return Err("Invalid assignment target".into());
                }
            }
        }
        Ok(lhs_expr)
    }

    fn equality(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut lhs_expr = self.comparision()?;
        while self.match_tokens(vec![BangEqual, EqualEqual]) {
            let op = self.previous().clone();
            let rhs_expr = self.comparision()?;
            lhs_expr = Expr::Binary {
                left: Box::from(lhs_expr),
                operator: op,
                right: Box::from(rhs_expr),
            };
        }
        Ok(lhs_expr)
    }

    fn comparision(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut lhs_expr = self.term()?;

        while self.match_tokens(vec![Greater, GreaterEqual, LessEqual, Less]) {
            let op = self.previous().clone();
            let rhs_expr = self.term()?;
            lhs_expr = Expr::Binary {
                left: Box::from(lhs_expr),
                operator: op,
                right: Box::from(rhs_expr),
            }
        }

        Ok(lhs_expr)
    }

    fn term(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut lhs_expr = self.factor()?;

        while self.match_tokens(vec![Minus, Plus]) {
            let op = self.previous().clone();
            let rhs_expr = self.factor()?;
            lhs_expr = Expr::Binary {
                left: Box::from(lhs_expr),
                operator: op,
                right: Box::from(rhs_expr),
            }
        }

        Ok(lhs_expr)
    }

    fn factor(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut lhs_expr = self.unary()?;

        while self.match_tokens(vec![Slash, Star]) {
            let op = self.previous().clone();
            let rhs_expr = self.unary()?;
            lhs_expr = Expr::Binary {
                left: Box::from(lhs_expr),
                operator: op,
                right: Box::from(rhs_expr),
            }
        }

        Ok(lhs_expr)
    }

    fn unary(&mut self) -> Result<Expr, Box<dyn Error>> {
        if self.match_tokens(vec![Minus, Bang]) {
            let op = self.previous().clone();
            let rhs_expr = self.unary()?;
            return Ok(Expr::Unary {
                operator: op,
                right: Box::from(rhs_expr),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Box<dyn Error>> {
        let token = self.peek();

        let result;
        match token.token_type {
            LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(RightParen, "Expected ')'")?;
                result = Expr::Grouping {
                    expression: Box::from(expr),
                };
            }
            Number | String_ | True | False | Nil => {
                result = Expr::Literal {
                    literal: LiteralValue::from_token(token),
                };
                self.advance();
            }
            Identifier => {
                result = Expr::Variable {
                    name: token.clone(),
                };
                self.advance();
            }

            _ => return Err(format!("{:?} is not a primary", self.peek()).into()),
        }
        Ok(result)
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token, Box<dyn Error>> {
        let token = self.peek();
        if token.token_type == token_type {
            self.advance();
            let token = self.previous();
            Ok(token.clone())
        } else {
            Err(msg.to_string().into())
        }
    }

    fn previous(&mut self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&mut self) -> bool {
        self.tokens[self.current].token_type == Eof
    }

    fn peek(&mut self) -> &Token {
        &self.tokens[self.current]
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn match_token(&mut self, token: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else if self.peek().token_type == token {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_tokens(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.match_token(token_type) {
                return true;
            }
        }
        false
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.peek().token_type == Eof {
                return;
            }
            match self.peek().token_type {
                Class | Func | Var | For | If | While | Print | Return => return,
                _ => (),
            }
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;
    use crate::Scanner;
    #[test]
    fn test_addition() -> Result<(), Box<dyn Error>> {
        let source = "1+2;";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse()?;

        println!("{:#?}", parsed_expr);
        //assert_eq!(string_expr, "(+ 1 2)");
        Ok(())
    }

    #[test]
    fn test_comparison() -> Result<(), Box<dyn Error>> {
        let source = "1+2 == 3+4;";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse()?;

        println!("{:#?}", parsed_expr);
        //assert_eq!(string_expr, "(== (+ 1 2) (+ 3 4))");
        Ok(())
    }

    #[test]
    fn test_factor() -> Result<(), Box<dyn Error>> {
        let source = "3-4*2;";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse()?;

        println!("{:#?}", parsed_expr);
        //assert_eq!(string_expr, "(- 3 (* 4 2))");
        Ok(())
    }

    #[test]
    fn test_eq_with_paren() -> Result<(), Box<dyn Error>> {
        let source = "1 == (2+2);";
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().unwrap();

        let mut parser = Parser::new(tokens);
        let parsed_expr = parser.parse()?;

        println!("{:#?}", parsed_expr);
        //assert_eq!(string_expr, "(== 1 (group (+ 2 2)))");
        Ok(())
    }
}
