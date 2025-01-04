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

#[derive(Debug)]
enum FunctionKind {
    Function,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    // The Main parse function that is called from outside
    // Converts the tokens into a array of statements
    // Returns errors together by storing them in a array
    pub fn parse(&mut self) -> Result<Vec<Stmt>, Box<dyn Error>> {
        let mut stmts = vec![];
        let mut errors = vec![];

        // go through all the tokens
        while !self.is_at_end() {
            // get a single statement
            let stmt = self.declaration();
            match stmt {
                Ok(s) => stmts.push(s),
                Err(e) => {
                    errors.push(e);
                    // If we get a error we need to move the pointer forward to where we can
                    // continue parsing
                    self.synchronize();
                }
            }
        }

        if errors.is_empty() {
            Ok(stmts)
        } else {
            // If u get errors report them together
            let mut err = String::new();
            for error in errors {
                err.push_str(format!("{}{}", &error.to_string(), "\n").as_str());
            }
            Err(err.into())
        }
    }

    // Matches the start of a statement to multiple branches
    fn declaration(&mut self) -> Result<Stmt, Box<dyn Error>> {
        if self.match_token(TokenType::Var) {
            self.var_declaration()
        } else if self.match_token(Func) {
            self.function(FunctionKind::Function)
        } else {
            self.statement()
        }
    }

    // Function declaration
    fn function(&mut self, kind: FunctionKind) -> Result<Stmt, Box<dyn Error>> {
        // Get the function name
        let token = self.consume(
            TokenType::Identifier,
            format!("Expected {:?} name", kind).as_str(),
        )?;
        // Check for the (
        self.consume(
            LeftParen,
            format!("Expected '(' after {:?} name", kind).as_str(),
        )?;

        let mut params = vec![];
        // Check for either no params
        if !self.check(RightParen) {
            loop {
                if params.len() >= 255 {
                    // Max length for params is 255
                    return Err(format!(
                        "Line {}: Cannot have more than 255 args",
                        self.peek().line_number
                    )
                    .into());
                }
                params.push(self.consume(Identifier, "Expected parameter name")?);
                // Need a comma after param
                if !self.match_token(Comma) {
                    break;
                }
            }
        }

        self.consume(RightParen, "Expected ')' after parameters")?;
        // Enter the function block
        self.consume(
            LeftBrace,
            format!("Expected '{}' before {:?} name", "{", kind).as_str(),
        )?;

        // The body of the function which is basically a block
        // Will return a array of statements
        let body = match self.block()? {
            Stmt::Block { stmts } => stmts,
            _ => panic!("Block statement parsed something that was not a block"),
        };

        // Return a function statement
        Ok(Stmt::Function {
            name: token,
            params,
            body,
        })
    }

    // Encountered the 'var' keyword
    fn var_declaration(&mut self) -> Result<Stmt, Box<dyn Error>> {
        // Get the variable name
        let token = self.consume(TokenType::Identifier, "Expect variable name.")?;

        // Check if the variable is initialized
        // var a; -> declaration
        // var a=1; -> initialized
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

    // Here we get the statements that have a lower presedence than in the declaration
    fn statement(&mut self) -> Result<Stmt, Box<dyn Error>> {
        if self.match_token(Print) {
            self.print_expression()
        } else if self.match_token(LeftBrace) {
            self.block()
        } else if self.match_token(If) {
            self.if_statement()
        } else if self.match_token(While) {
            self.while_statement()
        } else if self.match_token(For) {
            self.for_statement()
        } else {
            self.expression_statement()
        }
    }

    // For loop is syntactic sugar and uses while loop under the hood
    fn for_statement(&mut self) -> Result<Stmt, Box<dyn Error>> {
        self.consume(LeftParen, "Expect '(' after 'for'.")?;
        // Check if a variable is initialized, assigned a new val or is not given at all
        let initializer = if self.match_token(Semicolon) {
            None
        } else if self.match_token(Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        // Check if a condition exists or not
        let cond = if !self.check(Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(Semicolon, "Expect ';' after loop condition.")?;

        // Check if a increment exists or not
        let increment = if !self.check(RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(RightParen, "Expect ')' after for clauses.")?;

        // The body of a for loop is basically a block
        // We append the increment to the end of said block
        let body = if let Some(expr) = increment {
            let stmts = vec![
                Box::from(self.statement()?),
                Box::from(Stmt::Expression { expression: expr }),
            ];
            Stmt::Block { stmts }
        } else {
            self.statement()?
        };

        // If there is no condition we set it to True
        let cond = if let Some(s) = cond {
            s
        } else {
            Expr::Literal {
                literal: LiteralValue::True,
            }
        };

        // We create a while loop using the above block with the increment
        let mut body_while = Stmt::WhileLoop {
            cond,
            body: Box::from(body),
        };

        // If we have a increment we nest the while loop in another block and initalize the
        // initializer in the parent block
        if let Some(expr) = initializer {
            body_while = Stmt::Block {
                stmts: vec![Box::from(expr), Box::from(body_while)],
            };
        }

        Ok(body_while)
    }

    // While loop is basically a reoccouring block statement
    fn while_statement(&mut self) -> Result<Stmt, Box<dyn Error>> {
        self.consume(LeftParen, "Expect '(' after 'while'.")?;
        let cond = self.expression()?;
        self.consume(RightParen, "Expect ')' after condition.")?;
        // Should return a Block Statement
        let body = Box::from(self.statement()?);

        Ok(Stmt::WhileLoop { cond, body })
    }

    // Get the condition/predicate and then_branch and else_branch if it exists
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

        Ok(Stmt::IfElse {
            predicate,
            then_branch,
            else_branch,
        })
    }

    // Creates a array of statements till we reach a '}'
    fn block(&mut self) -> Result<Stmt, Box<dyn Error>> {
        let mut stmts = vec![];

        while !self.check(RightBrace) && !self.is_at_end() {
            let stmt = self.declaration()?;
            stmts.push(Box::from(stmt));
        }

        self.consume(RightBrace, "Expect '}' after block.")?;

        Ok(Stmt::Block { stmts })
    }

    // Printing branch
    fn print_expression(&mut self) -> Result<Stmt, Box<dyn Error>> {
        let val = self.expression()?;
        self.consume(Semicolon, "Expected ';' after value")?;
        Ok(Stmt::Print { expression: val })
    }

    // Normal expression
    fn expression_statement(&mut self) -> Result<Stmt, Box<dyn Error>> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::Expression { expression: expr })
    }

    fn expression(&mut self) -> Result<Expr, Box<dyn Error>> {
        self.assignment()
    }

    // Assigning values to variables
    fn assignment(&mut self) -> Result<Expr, Box<dyn Error>> {
        let lhs_expr = self.or()?;

        // Is the variable initialized
        if self.match_token(Equal) {
            let _eq = self.previous();
            // Get the RHS
            let rhs_expr = self.assignment()?;
            match lhs_expr {
                // Create the Expression
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

    // OR logical operator
    fn or(&mut self) -> Result<Expr, Box<dyn Error>> {
        let lhs_expr = self.and()?;

        if self.match_token(Or) {
            let op = self.previous().clone();
            let rhs_expr = self.and()?;
            return Ok(Expr::Logical {
                left: Box::from(lhs_expr),
                operator: op,
                right: Box::from(rhs_expr),
            });
        }
        Ok(lhs_expr)
    }

    // AND logical operator
    fn and(&mut self) -> Result<Expr, Box<dyn Error>> {
        let lhs_expr = self.equality()?;

        if self.match_token(And) {
            let op = self.previous().clone();
            let rhs_expr = self.equality()?;
            return Ok(Expr::Logical {
                left: Box::from(lhs_expr),
                operator: op,
                right: Box::from(rhs_expr),
            });
        }
        Ok(lhs_expr)
    }

    // Creates Expression for == or !=
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

    // Creates Expr for >, <, >=, <=
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

    // Resolves binary operations such as - or +
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

    // Resolves binay operators such as / or *
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

    // Unary operators
    fn unary(&mut self) -> Result<Expr, Box<dyn Error>> {
        if self.match_tokens(vec![Minus, Bang]) {
            let op = self.previous().clone();
            let rhs_expr = self.unary()?;
            return Ok(Expr::Unary {
                operator: op,
                right: Box::from(rhs_expr),
            });
        }
        self.call()
    }

    // Function call
    fn call(&mut self) -> Result<Expr, Box<dyn Error>> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(LeftParen) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    // Parse a function call
    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Box<dyn Error>> {
        let mut args = vec![];

        // Get the arguments
        if !self.check(RightParen) {
            loop {
                let arg = self.expression()?;
                if args.len() >= 255 {
                    return Err(format!(
                        "Line {}: Cannot have more than 255 args",
                        self.peek().line_number
                    )
                    .into());
                }
                args.push(arg);
                if !self.match_token(Comma) {
                    break;
                }
            }
        }

        let paren = self.consume(RightParen, "Expexted ')' after arguments")?;
        // Create a Call Expression
        Ok(Expr::Call {
            callee: Box::from(callee),
            paren,
            args,
        })
    }

    // primaries such as True, False, Number, String etc
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

    // consume the given token or return a error if the token does not match the expected one
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

    // Return a previous token
    fn previous(&mut self) -> &Token {
        &self.tokens[self.current - 1]
    }

    // Check if the source is finished
    fn is_at_end(&mut self) -> bool {
        self.tokens[self.current].token_type == Eof
    }

    // Return the current token
    fn peek(&mut self) -> &Token {
        &self.tokens[self.current]
    }

    // Check if the given token is the current token
    fn check(&mut self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    // matches a given token and then advances to the next
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

    // Match token buut for a array
    fn match_tokens(&mut self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.match_token(token_type) {
                return true;
            }
        }
        false
    }

    // Go ahead one token
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    // Sync up to the code if we hit a error
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
