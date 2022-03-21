use bang::print;

use ahash::AHashMap as HashMap;
use clap::{Arg, Command};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::{fs, rc::Rc};

fn read_file(filename: &str) -> String {
  if let Ok(file) = fs::read_to_string(filename) {
    file
  } else {
    println!("Problem reading file: {}", filename);
    String::new()
  }
}

fn compile(source: &str) -> Result<bang::Chunk, bang::Diagnostic> {
  let tokens = bang::tokenize(source);
  let ast = bang::parse(source, &tokens)?;

  bang::compile(source, &ast)
}

fn interpret(source: &str) -> Result<HashMap<Rc<str>, bang::Value>, bang::Diagnostic> {
  let chunk = compile(source)?;

  bang::run(chunk)
}

fn repl() {
  let mut rl = Editor::<()>::new();
  let mut vm = bang::VM::new();

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
    .get_matches();

  if let Some((command @ ("lint" | "run" | "tokens" | "ast" | "bytecode" | "format"), subcommand)) =
    app.subcommand()
  {
    let filename = subcommand.value_of("file").unwrap();
    let source = read_file(filename);
    let tokens = bang::tokenize(&source);

    if source.is_empty() {
      return;
    }

    match command {
      "run" => match interpret(&source) {
        Ok(_) => {}
        Err(details) => print::error(filename, &source, details),
      },
      "lint" => match bang::parse(&source, &tokens) {
        Ok(ast) => {
          for lint in bang::lint(&source, &ast) {
            print::warning(filename, &source, lint);
          }
        }
        Err(details) => print::error(filename, &source, details),
      },
      "tokens" => print::tokens(&source, &tokens),
      "ast" => match bang::parse(&source, &tokens) {
        Ok(ast) => print::ast(&source, &ast),
        Err(details) => print::error(filename, &source, details),
      },
      "bytecode" => match compile(&source) {
        Ok(chunk) => print::chunk(&chunk),
        Err(details) => print::error(filename, &source, details),
      },
      "format" => match bang::parse(&source, &tokens) {
        Ok(ast) => {
          let new_source = bang::format(&source, &ast);

          if subcommand.is_present("dryrun") {
            println!("{}", new_source);
          } else if new_source != source {
            fs::write(filename, new_source).unwrap();
          } else {
            println!("'{}' already matches the Bang format style!", filename);
          }
        }
        Err(details) => print::error(filename, &source, details),
      },
      _ => unreachable!(),
    }
  } else {
    println!("Bang! ({})", version);
    repl();
  }
}
