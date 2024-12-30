mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
use interpreter::Interpreter;
use parser::Parser;
//use stmt::Stmt;

use crate::scanner::*;

use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::process::exit;

fn run_file(path: &str) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let mut interpreter: Interpreter = Interpreter::new();
    run(&mut interpreter, &contents)?;
    //for line in contents.split(";") {
    //    run(&mut interpreter,&line)?;
    //}
    Ok(())
}

fn run(interpreter: &mut Interpreter, contents: &str) -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new(contents);
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);

    let stmts = parser.parse()?;
    interpreter.interpret(stmts)?;

    Ok(())
}

fn run_prompt() -> Result<(), Box<dyn Error>> {
    let mut interpreter: Interpreter = Interpreter::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut buffer = String::new();
        stdin.read_line(&mut buffer)?;
        if buffer.trim() == "exit" || buffer.trim() == "" {
            break;
        }
        match run(&mut interpreter, &buffer) {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        }
        //println!("Echo: {}", buffer);
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        if let Err(e) = run_prompt() {
            println!("Error: {}", e);
            exit(1);
        }
    } else if args.len() == 2 {
        if let Err(e) = run_file(&args[1]) {
            println!("Error: {}", e);
            exit(1);
        }
    } else {
        println!("Usage: script");
        println!("\tOR");
        println!("Usage: script [file path]");
        exit(64);
    }
    //dbg!(args);
}
