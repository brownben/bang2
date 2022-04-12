use clap::{Arg, Command};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;

use bang_language::{
  compile as compile_ast, parse, print, run, tokenize, Chunk, Diagnostic, VMGlobals, VM,
};
use bang_tools::{format, lint, typecheck};

mod print_diagnostic;
use print_diagnostic::{error as print_error, warning as print_warning};

fn read_file(filename: &str) -> String {
  if let Ok(file) = fs::read_to_string(filename) {
    file
  } else {
    println!("Problem reading file: {}", filename);
    String::new()
  }
}

fn compile(source: &str) -> Result<Chunk, Diagnostic> {
  let tokens = tokenize(source);
  let ast = parse(source, &tokens)?;

  compile_ast(source, &ast)
}

fn interpret(source: &str) -> Result<VMGlobals, Diagnostic> {
  let chunk = compile(source)?;

  run(chunk)
}

fn repl() {
  let mut rl = Editor::<()>::new();
  let mut vm = VM::new();

  loop {
    let readline = rl.readline("> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(&line);

        match compile(&format!("{}\n", line)) {
          Ok(chunk) => match vm.run(chunk) {
            Ok(_) => {}
            Err(error) => print_error("REPL", &line, error),
          },
          Err(details) => print_error("REPL", &line, details),
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
      Command::new("run")
        .about("Execute a Bang program")
        .arg(Arg::new("file").help("The file to run").required(true)),
    )
    .subcommand(
      Command::new("lint")
        .about("Run linter on a bang file")
        .arg(Arg::new("file").help("The file to lint").required(true)),
    )
    .subcommand(
      Command::new("tokens")
        .about("Display the Tokens for a file")
        .arg(
          Arg::new("file")
            .help("The file scan for tokens")
            .required(true),
        ),
    )
    .subcommand(
      Command::new("ast")
        .about("Display the Abstract Syntax Tree for a file")
        .arg(Arg::new("file").help("The file to parse").required(true)),
    )
    .subcommand(
      Command::new("bytecode")
        .about("Display the Bytecode from a file")
        .arg(Arg::new("file").help("The file to compile").required(true)),
    )
    .subcommand(
      Command::new("format")
        .alias("fmt")
        .about("Format a bang file")
        .arg(Arg::new("file").help("The file to format").required(true))
        .arg(
          Arg::new("dryrun")
            .long("dryrun")
            .help("Preview the results of the formatting"),
        ),
    )
    .subcommand(
      Command::new("typecheck")
        .about("Run typechecker on on a file")
        .arg(
          Arg::new("file")
            .help("The file to typecheck")
            .required(true),
        ),
    )
    .get_matches();

  if let Some((
    command @ ("lint" | "run" | "tokens" | "ast" | "bytecode" | "format" | "typecheck"),
    subcommand,
  )) = app.subcommand()
  {
    let filename = subcommand.value_of("file").unwrap();
    let source = read_file(filename);
    let tokens = tokenize(&source);

    if source.is_empty() {
      return;
    }

    match command {
      "run" => match interpret(&source) {
        Ok(_) => {}
        Err(details) => print_error(filename, &source, details),
      },
      "lint" => match parse(&source, &tokens) {
        Ok(ast) => {
          for lint in lint(&source, &ast) {
            print_warning(filename, &source, lint);
          }
        }
        Err(details) => print_error(filename, &source, details),
      },
      "tokens" => print::tokens(&source, &tokens),
      "ast" => match parse(&source, &tokens) {
        Ok(ast) => print::ast(&source, &ast),
        Err(details) => print_error(filename, &source, details),
      },
      "bytecode" => match compile(&source) {
        Ok(chunk) => print::chunk(&chunk),
        Err(details) => print_error(filename, &source, details),
      },
      "format" => match parse(&source, &tokens) {
        Ok(ast) => {
          let new_source = format(&source, &ast);

          if subcommand.is_present("dryrun") {
            println!("{}", new_source);
          } else if new_source != source {
            fs::write(filename, new_source).unwrap();
          } else {
            println!("'{}' already matches the Bang format style!", filename);
          }
        }
        Err(details) => print_error(filename, &source, details),
      },
      "typecheck" => match parse(&source, &tokens) {
        Ok(ast) => {
          for error in typecheck(&source, &ast) {
            print_error(filename, &source, error);
          }
        }
        Err(details) => print_error(filename, &source, details),
      },
      _ => unreachable!(),
    }
  } else {
    println!("Bang! ({})", version);
    repl();
  }
}
