mod ast;
mod builtin;
mod chunk;
mod compiler;
mod error;
mod linter;
mod parser;
mod print;
mod scanner;
mod token;
mod value;
mod vm;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;
extern crate clap;
use clap::{App, Arg, SubCommand};

fn read_file(filename: &str) -> String {
  if let Ok(file) = fs::read_to_string(filename) {
    file
  } else {
    print::error(&format!("Problem reading file: {}", filename));
    String::new()
  }
}

fn compile(source: &str) -> Result<chunk::Chunk, error::CompileError> {
  let ast = parser::parse(source)?;
  compiler::compile(ast)
}

fn repl() {
  let mut rl = Editor::<()>::new();
  let mut vm = vm::VM::new();
  builtin::define_globals(&mut vm);

  loop {
    let readline = rl.readline("> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(&line);

        match compile(&format!("{}\n", line)) {
          Ok(chunk) => match vm.run(value::Function::script(chunk)) {
            Ok(_) => {}
            Err(error) => print::runtime_error("REPL", &line, &error),
          },
          Err(details) => print::compile_error("REPL", &line, &details),
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
  let app = App::new("bang")
    .version(version)
    .subcommand(SubCommand::with_name("").about("Open a REPL"))
    .subcommand(
      SubCommand::with_name("run")
        .about("Execute a Bang program")
        .arg(
          Arg::with_name("file")
            .help("The file to run")
            .required(true)
            .index(1),
        ),
    )
    .subcommand(
      SubCommand::with_name("lint")
        .about("Run linter on a bang file")
        .arg(
          Arg::with_name("file")
            .help("The file to lint")
            .required(true)
            .index(1),
        ),
    )
    .subcommand(
      SubCommand::with_name("tokens")
        .about("Display the Tokens for a file")
        .arg(
          Arg::with_name("file")
            .help("The file scan for tokens")
            .required(true)
            .index(1),
        ),
    )
    .subcommand(
      SubCommand::with_name("ast")
        .about("Display the Abstract Syntax Tree for a file")
        .arg(
          Arg::with_name("file")
            .help("The file to parse")
            .required(true)
            .index(1),
        ),
    )
    .subcommand(
      SubCommand::with_name("bytecode")
        .about("Display the Bytecode from a file")
        .arg(
          Arg::with_name("file")
            .help("The file to compile")
            .required(true)
            .index(1),
        ),
    )
    .get_matches();

  match app.subcommand() {
    (command @ ("lint" | "run" | "tokens" | "ast" | "bytecode"), Some(subcommand)) => {
      let filename = subcommand.value_of("file").unwrap();
      let source = read_file(filename);

      match command {
        "run" => match compile(&source) {
          Ok(chunk) => match vm::run(chunk) {
            Ok(_) => {}
            Err(error) => print::runtime_error(filename, &source, &error),
          },
          Err(details) => print::compile_error(filename, &source, &details),
        },
        "lint" => match parser::parse(&source) {
          Ok(ast) => linter::lint(&ast)
            .iter()
            .for_each(|warning| print::lint_warning(filename, &source, warning)),
          Err(details) => print::compile_error(filename, &source, &details),
        },
        "tokens" => print::tokens(&source),
        "ast" => match parser::parse(&source) {
          Ok(ast) => print::ast(&ast),
          Err(details) => print::compile_error(filename, &source, &details),
        },
        "bytecode" => match compile(&source) {
          Ok(chunk) => print::chunk(&chunk, filename),
          Err(details) => print::compile_error(filename, &source, &details),
        },
        _ => unreachable!(),
      }
    }
    _ => {
      println!("Bang! ({})\n", version);
      repl()
    }
  }
}
