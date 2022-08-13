use bang_interpreter::{compile, Chunk, RuntimeError, VM};
use bang_std::StdContext as Context;
use bang_syntax::parse;
use bang_tools::{format, lint, typecheck};
use clap::{Arg, Command};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;

mod print;

fn read_file(filename: &str) -> String {
  if let Ok(file) = fs::read_to_string(filename) {
    file
  } else {
    println!("Problem reading file: {}", filename);
    String::new()
  }
}

fn run(chunk: &Chunk) -> Result<(), RuntimeError> {
  let mut vm = VM::new(&Context);
  vm.run(chunk)
}

fn repl() {
  let mut rl = Editor::<()>::new();
  let mut vm = VM::new(&Context);

  loop {
    let readline = rl.readline("> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(&line);

        match compile(&format!("print({})\n", line), &Context) {
          Ok(chunk) => match vm.run(&chunk) {
            Ok(_) => {}
            Err(error) => print::runtime_error("REPL", &line, error),
          },
          Err(details) => print::error("REPL", &line, &details),
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
    command @ ("lint" | "run" | "ast" | "bytecode" | "format" | "typecheck"),
    subcommand,
  )) = app.subcommand()
  {
    let filename = subcommand.value_of("file").unwrap();
    let source = read_file(filename);

    if source.is_empty() {
      return;
    }

    match command {
      "run" => match compile(&source, &Context) {
        Ok(chunk) => match run(&chunk) {
          Ok(_) => {}
          Err(error) => print::runtime_error(filename, &source, error),
        },

        Err(details) => print::error(filename, &source, &details),
      },
      "lint" => match parse(&source) {
        Ok(ast) => {
          for lint in lint(&source, &ast) {
            print::warning(filename, &source, lint);
          }
        }
        Err(details) => print::error(filename, &source, &details),
      },
      "ast" => match parse(&source) {
        Ok(ast) => print::ast(&source, &ast),
        Err(details) => print::error(filename, &source, &details),
      },
      "bytecode" => match compile(&source, &Context) {
        Ok(chunk) => print::chunk(&chunk),
        Err(details) => print::error(filename, &source, &details),
      },
      "format" => match parse(&source) {
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
        Err(details) => print::error(filename, &source, &details),
      },
      "typecheck" => match parse(&source) {
        Ok(ast) => {
          for error in typecheck(&source, &ast) {
            print::error(filename, &source, &error);
          }
        }
        Err(details) => print::error(filename, &source, &details),
      },
      _ => unreachable!(),
    }
  } else {
    println!("Bang! ({})", version);
    repl();
  }
}
