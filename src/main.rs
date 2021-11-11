use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::env;
use std::fs;
use std::process::exit;

mod chunk;
mod compiler;
mod error;
mod scanner;
mod value;
mod vm;

use vm::{InterpreterResult, VM};

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(),
        2 => run_file(&args[1]),
        _ => {
            println!("Usage: bang [file]");
            exit(64);
        }
    };
}

fn repl() {
    let mut rl = Editor::<()>::new();
    let mut vm = VM::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let line = line + "\n";
                vm.interpret(&line, String::from("repl"));
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn run_file(filename: &str) {
    if let Ok(file) = fs::read_to_string(filename) {
        let mut vm = VM::new();
        let result = vm.interpret(&file, String::from(filename));

        match result {
            InterpreterResult::CompileError => exit(65),
            InterpreterResult::RuntimeError => exit(70),
            _ => exit(0),
        }
    } else {
        println!("Problem reading file '{}'", filename);
        exit(74);
    }
}
