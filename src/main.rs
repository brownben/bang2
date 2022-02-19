#![feature(box_patterns)]

mod ast;
mod builtins;
mod chunk;
mod compiler;
mod diagnostic;
mod linter;
mod parser;
mod print;
mod tokens;
mod value;
mod vm;

use clap::{Arg, Command};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::{collections::HashMap, fs, rc::Rc};

fn read_file(filename: &str) -> String {
  if let Ok(file) = fs::read_to_string(filename) {
    file
  } else {
    println!("Problem reading file: {}", filename);
    String::new()
  }
}

fn compile(source: &str) -> Result<chunk::Chunk, diagnostic::Diagnostic> {
  let tokens = tokens::tokenize(source);
  let ast = parser::parse(&tokens)?;

  compiler::compile(&ast)
}

fn interpret(source: &str) -> Result<HashMap<Rc<str>, value::Value>, diagnostic::Diagnostic> {
  let chunk = compile(source)?;

  vm::run(chunk)
}

fn repl() {
  let mut rl = Editor::<()>::new();
  let mut vm = vm::VM::new();
  builtins::define_globals(&mut vm);

  loop {
    let readline = rl.readline("> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(&line);

        match compile(&format!("{}\n", line)) {
          Ok(chunk) => match vm.run(chunk) {
            Ok(_) => {}
            Err(error) => print::error("REPL", &line, error),
          },
          Err(details) => print::error("REPL", &line, details),
        }
      }
      Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
        break;
      }
      Err(err) => {
        println!("Error: {:?}", err);
        break;
      }
    }
  }
}

fn main() {
  let version = "v2.0-alpha";
  let app = Command::new("bang")
    .version(version)
    .subcommand(Command::new("").about("Open a REPL"))
    .subcommand(
      Command::new("run").about("Execute a Bang program").arg(
        Arg::new("file")
          .help("The file to run")
          .required(true)
          .index(1),
      ),
    )
    .subcommand(
      Command::new("lint").about("Run linter on a bang file").arg(
        Arg::new("file")
          .help("The file to lint")
          .required(true)
          .index(1),
      ),
    )
    .subcommand(
      Command::new("tokens")
        .about("Display the Tokens for a file")
        .arg(
          Arg::new("file")
            .help("The file scan for tokens")
            .required(true)
            .index(1),
        ),
    )
    .subcommand(
      Command::new("ast")
        .about("Display the Abstract Syntax Tree for a file")
        .arg(
          Arg::new("file")
            .help("The file to parse")
            .required(true)
            .index(1),
        ),
    )
    .subcommand(
      Command::new("bytecode")
        .about("Display the Bytecode from a file")
        .arg(
          Arg::new("file")
            .help("The file to compile")
            .required(true)
            .index(1),
        ),
    )
    .get_matches();

  if let Some((command @ ("lint" | "run" | "tokens" | "ast" | "bytecode"), subcommand)) =
    app.subcommand()
  {
    let filename = subcommand.value_of("file").unwrap();
    let source = read_file(filename);
    let tokens = tokens::tokenize(&source);

    match command {
      "run" => match interpret(&source) {
        Ok(_) => {}
        Err(details) => print::error(filename, &source, details),
      },
      "lint" => match parser::parse(&tokens) {
        Ok(ast) => {
          for lint in linter::lint(&ast) {
            print::warning(filename, &source, lint);
          }
        }
        Err(details) => print::error(filename, &source, details),
      },
      "tokens" => print::tokens(&tokens),
      "ast" => match parser::parse(&tokens) {
        Ok(ast) => print::ast(&ast),
        Err(details) => print::error(filename, &source, details),
      },
      "bytecode" => match compile(&source) {
        Ok(chunk) => print::chunk(&chunk, filename),
        Err(details) => print::error(filename, &source, details),
      },
      _ => unreachable!(),
    }
  } else {
    println!("Bang! ({})\n", version);
    repl();
  }
}
