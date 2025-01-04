use crate::TokenType::*;
use core::fmt;
use std::{collections::HashMap, error::Error, string::String};

// Scan through the buffer given and give out tokens
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<&'static str, TokenType>,
}

//Helper functions
fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

fn is_alpha(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_alpha_num(ch: char) -> bool {
    is_alpha(ch) || is_digit(ch)
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            keywords: HashMap::from([
                ("and", And),
                ("or", Or),
                ("class", Class),
                ("else", Else),
                ("if", If),
                ("true", True),
                ("false", False),
                ("for", For),
                ("nil", Nil),
                ("print", Print),
                ("return", Return),
                ("func", Func),
                ("this", This),
                ("while", While),
                ("super", Super),
                ("var", Var),
            ]),
        }
    }

    // Main scanner function that is invoked from the main
    // Returns a list of tokens in the whole buffer given
    // Stores a list of errors and returns them together in a long list
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Box<dyn Error>> {
        let mut errors = vec![];
        // While not at the end of the file keep on going
        while !self.is_at_end() {
            // shift the start index to where the previous run ended
            self.start = self.current;
            // scann tokens in line
            // if err store it to report together
            if let Err(e) = self.scan_token() {
                errors.push(e)
            }
        }
        // After scanning everything push a EOF Token at the end
        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line_number: self.line,
        });

        // If any error print all of them together
        if !errors.is_empty() {
            let mut joined = "".to_string();
            errors.iter().for_each(|error| {
                joined.push_str(format!("{}", error).as_str());
                joined.push('\n');
            });
            return Err(joined.into());
        }
        Ok(self.tokens.clone())
    }

    // Check if we have exceded the length of the document/source
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    // Core scanner function where it branches acording to the syntax
    fn scan_token(&mut self) -> Result<(), Box<dyn Error>> {
        let c = self.advance();

        match c {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '+' => self.add_token(Plus),
            '-' => self.add_token(Minus),
            ';' => self.add_token(Semicolon),
            '*' => self.add_token(Star),

            '!' => {
                let token = if self.char_match('=') {
                    BangEqual
                } else {
                    Bang
                };
                self.add_token(token);
            }
            '=' => {
                let token = if self.char_match('=') {
                    EqualEqual
                } else {
                    Equal
                };
                self.add_token(token);
            }
            '>' => {
                let token = if self.char_match('=') {
                    GreaterEqual
                } else {
                    Greater
                };
                self.add_token(token);
            }
            '<' => {
                let token = if self.char_match('=') {
                    LessEqual
                } else {
                    Less
                };
                self.add_token(token);
            }

            '/' => {
                if self.char_match('/') {
                    loop {
                        if self.peek() == '\n' || self.is_at_end() {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    self.add_token(Slash);
                };
            }

            '"' => {
                self.string_literal()?;
            }

            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,

            c => {
                if is_digit(c) {
                    self.number()?;
                } else if is_alpha(c) {
                    self.identifier()?;
                } else {
                    return Err(format!("Unrecognised char {} at line {}", c, self.line).into());
                }
            }
        }
        Ok(())
    }

    // This is called when we get a word that does not start with a number
    fn identifier(&mut self) -> Result<(), Box<dyn Error>> {
        // Keep moving the current pointer ahead till we see alphabets
        while is_alpha_num(self.peek()) {
            self.advance();
        }

        // Get the identifier ranging from start to current
        let substring = &self.source[self.start..self.current];
        // Check if the identifier is a reserved keyword
        let token_type = match self.keywords.get(substring) {
            Some(e) => e.clone(),
            None => Identifier,
        };

        self.add_token(token_type);
        Ok(())
    }

    // Run like the identifier but when the word starts with a number
    fn number(&mut self) -> Result<(), Box<dyn Error>> {
        // Keep moving the current pointer ahead till we see digits
        while is_digit(self.peek()) {
            self.advance();
        }

        // Check if floating point is followed by a number
        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();
            // Get the number following the dot
            while is_digit(self.peek()) {
                self.advance();
            }
        }

        // get the int or float as a string
        let s = &self.source.as_str()[self.start..self.current];
        // pasre it to f64
        match s.parse::<f64>() {
            Ok(v) => {
                // All numbers are stored as float
                self.add_token_lit(Number, Some(LiteralValue::FloatValue(v)));
            }
            Err(_) => return Err(format!("Failed to parse number at line {}", self.line).into()),
        }
        Ok(())
    }

    // Helper function to check if current char matches a given char and moves current by one
    fn char_match(&mut self, c: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] as char != c {
            false
        } else {
            self.current += 1;
            true
        }
    }

    // Called when we encounter '"'
    fn string_literal(&mut self) -> Result<(), Box<dyn Error>> {
        // Keep on going till the source ends or u find the closeing '"'
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        self.advance();
        // If we reach the end the string was not terminated
        if self.is_at_end() {
            return Err("String is not terminated".into());
        }
        // Get the literal as a string and then convert it into a LiteralValue enum
        let literal = &self.source.as_str()[self.start + 1..self.current - 1];
        let literal = LiteralValue::StringValue(literal.to_string());
        self.add_token_lit(String_, Some(literal));
        Ok(())
    }

    // Return the char after the current pointer
    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.as_bytes()[self.current] as char
    }

    // Returns the char after peek if it does not encounter the end
    fn peek_next(&self) -> char {
        if self.current + 1 > self.source.len() {
            '\0'
        } else {
            self.source.as_bytes()[self.current + 1] as char
        }
    }

    // Add a token with the None LiteralValue
    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_lit(token_type, None);
    }

    // Add a token to the struct tokens vector
    fn add_token_lit(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        let text = &self.source.as_str()[self.start..self.current];
        self.tokens.push(Token {
            token_type,
            lexeme: text.to_string(),
            literal,
            line_number: self.line,
        })
    }

    // return current char and increment the pointer by 1
    fn advance(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        let c = self.source.as_bytes()[self.current];
        self.current += 1;
        c as char
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    Comma,
    Dot,
    Plus,
    Minus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    EqualEqual,

    Identifier,
    String_,
    Number,

    And,
    Or,
    True,
    False,
    Class,
    If,
    Else,
    Func,
    For,
    While,
    Nil,
    Print,
    Return,
    Super,
    This,
    Var,

    Eof,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum LiteralValue {
    FloatValue(f64),
    StringValue(String),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub line_number: usize,
}

impl Token {
    #[allow(clippy::inherent_to_string, dead_code)]
    pub fn to_string(&self) -> String {
        format!("{} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_single_char_tokens() -> Result<(), Box<dyn Error>> {
        let source = "(){}=/-+*.,;";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens()?;

        assert_eq!(scanner.tokens.len(), source.len() + 1);
        assert_eq!(scanner.tokens[0].token_type, LeftParen);
        assert_eq!(scanner.tokens[1].token_type, RightParen);
        assert_eq!(scanner.tokens[2].token_type, LeftBrace);
        assert_eq!(scanner.tokens[3].token_type, RightBrace);
        assert_eq!(scanner.tokens[4].token_type, Equal);
        assert_eq!(scanner.tokens[5].token_type, Slash);
        assert_eq!(scanner.tokens[6].token_type, Minus);
        assert_eq!(scanner.tokens[7].token_type, Plus);
        assert_eq!(scanner.tokens[8].token_type, Star);
        assert_eq!(scanner.tokens[9].token_type, Dot);
        assert_eq!(scanner.tokens[10].token_type, Comma);
        assert_eq!(scanner.tokens[11].token_type, Semicolon);
        assert_eq!(scanner.tokens[12].token_type, Eof);

        Ok(())
    }

    #[test]
    fn handle_double_char_tokens() -> Result<(), Box<dyn Error>> {
        let source = "== >= <= != // this is a comment";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens()?;

        assert_eq!(scanner.tokens.len(), 5);
        assert_eq!(scanner.tokens[0].token_type, EqualEqual);
        assert_eq!(scanner.tokens[1].token_type, GreaterEqual);
        assert_eq!(scanner.tokens[2].token_type, LessEqual);
        assert_eq!(scanner.tokens[3].token_type, BangEqual);
        assert_eq!(scanner.tokens[4].token_type, Eof);

        Ok(())
    }

    #[test]
    fn check_is_digit() -> Result<(), Box<dyn Error>> {
        assert_eq!(is_digit('0'), true);
        assert_eq!(is_digit('1'), true);
        assert_eq!(is_digit('2'), true);
        assert_eq!(is_digit('3'), true);
        assert_eq!(is_digit('4'), true);
        assert_eq!(is_digit('5'), true);
        assert_eq!(is_digit('6'), true);
        assert_eq!(is_digit('7'), true);
        assert_eq!(is_digit('8'), true);
        assert_eq!(is_digit('9'), true);
        assert_eq!(is_digit('i'), false);
        Ok(())
    }

    #[test]
    fn check_is_alpha() -> Result<(), Box<dyn Error>> {
        assert_eq!(is_alpha('a'), true);
        assert_eq!(is_alpha('z'), true);
        assert_eq!(is_alpha('A'), true);
        assert_eq!(is_alpha('Z'), true);
        assert_eq!(is_alpha('-'), false);
        assert_eq!(is_alpha('f'), true);
        assert_eq!(is_alpha('F'), true);
        Ok(())
    }

    #[test]
    fn string_literal_test() -> Result<(), Box<dyn Error>> {
        let source = "\"Hello world\" ";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens()?;

        assert_eq!(scanner.tokens.len(), 2);
        assert_eq!(scanner.tokens[0].token_type, String_);
        assert_eq!(scanner.tokens[1].token_type, Eof);

        Ok(())
    }

    #[test]
    fn string_literal_multilind_test() -> Result<(), Box<dyn Error>> {
        let source = "\"Hello\nWorld\"\n";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens()?;

        assert_eq!(scanner.tokens.len(), 2);
        assert_eq!(scanner.tokens[0].token_type, String_);
        assert_eq!(scanner.tokens[1].token_type, Eof);

        Ok(())
    }

    #[test]
    fn number_literal_test() -> Result<(), Box<dyn Error>> {
        let source = "123.321\n432432.43242\n5.\n1\n.1";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens()?;

        assert_eq!(scanner.tokens.len(), 8);
        assert_eq!(scanner.tokens[0].token_type, Number);
        assert_eq!(scanner.tokens[1].token_type, Number);
        assert_eq!(scanner.tokens[2].token_type, Number);
        assert_eq!(scanner.tokens[3].token_type, Dot);
        assert_eq!(scanner.tokens[4].token_type, Number);
        assert_eq!(scanner.tokens[5].token_type, Dot);
        assert_eq!(scanner.tokens[6].token_type, Number);
        assert_eq!(scanner.tokens[7].token_type, Eof);

        Ok(())
    }

    #[test]
    fn identifier_test() -> Result<(), Box<dyn Error>> {
        let source = "hello this_ is a var_ and or class else if true false for nil print return func this while super var";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens()?;

        assert_eq!(scanner.tokens.len(), 22);
        assert_eq!(scanner.tokens[0].token_type, Identifier);
        assert_eq!(scanner.tokens[1].token_type, Identifier);
        assert_eq!(scanner.tokens[2].token_type, Identifier);
        assert_eq!(scanner.tokens[3].token_type, Identifier);
        assert_eq!(scanner.tokens[4].token_type, Identifier);
        assert_eq!(scanner.tokens[5].token_type, And);
        assert_eq!(scanner.tokens[6].token_type, Or);
        assert_eq!(scanner.tokens[7].token_type, Class);
        assert_eq!(scanner.tokens[8].token_type, Else);
        assert_eq!(scanner.tokens[9].token_type, If);
        assert_eq!(scanner.tokens[10].token_type, True);
        assert_eq!(scanner.tokens[11].token_type, False);
        assert_eq!(scanner.tokens[12].token_type, For);
        assert_eq!(scanner.tokens[13].token_type, Nil);
        assert_eq!(scanner.tokens[14].token_type, Print);
        assert_eq!(scanner.tokens[15].token_type, Return);
        assert_eq!(scanner.tokens[16].token_type, Func);
        assert_eq!(scanner.tokens[17].token_type, This);
        assert_eq!(scanner.tokens[18].token_type, While);
        assert_eq!(scanner.tokens[19].token_type, Super);
        assert_eq!(scanner.tokens[20].token_type, Var);
        assert_eq!(scanner.tokens[21].token_type, Eof);

        Ok(())
    }

    #[test]
    fn full_test() -> Result<(), Box<dyn Error>> {
        let source = "var x = 10;\nwhile x>1 { print(\"hello\"); }";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens()?;

        assert_eq!(scanner.tokens.len(), 17);
        assert_eq!(scanner.tokens[0].token_type, Var);
        assert_eq!(scanner.tokens[1].token_type, Identifier);
        assert_eq!(scanner.tokens[2].token_type, Equal);
        assert_eq!(scanner.tokens[3].token_type, Number);
        assert_eq!(scanner.tokens[4].token_type, Semicolon);
        assert_eq!(scanner.tokens[5].token_type, While);
        assert_eq!(scanner.tokens[6].token_type, Identifier);
        assert_eq!(scanner.tokens[7].token_type, Greater);
        assert_eq!(scanner.tokens[8].token_type, Number);
        assert_eq!(scanner.tokens[9].token_type, LeftBrace);
        assert_eq!(scanner.tokens[10].token_type, Print);
        assert_eq!(scanner.tokens[11].token_type, LeftParen);
        assert_eq!(scanner.tokens[12].token_type, String_);
        assert_eq!(scanner.tokens[13].token_type, RightParen);
        assert_eq!(scanner.tokens[14].token_type, Semicolon);
        assert_eq!(scanner.tokens[15].token_type, RightBrace);
        assert_eq!(scanner.tokens[16].token_type, Eof);

        Ok(())
    }
}
