mod bang {
  pub use bang_interpreter::*;
  pub use bang_std::*;
  pub use bang_syntax::*;
  pub use bang_tools::*;
}
mod helpers;
mod print;

use clap::{Arg, Command};
use helpers::{compile, get_filename, parse, read_file, run};
use std::fs;

const VERSION: &str = "v2.0-alpha";

fn main() {
  let app = Command::new("bang")
    .version(VERSION)
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
    .subcommand(
      Command::new("print")
        .about("Print debugging information")
        .subcommand_required(true)
        .subcommand(
          Command::new("ast")
            .about("Display the Abstract Syntax Tree for a file")
            .arg(Arg::new("file").help("The file to parse").required(true)),
        )
        .subcommand(
          Command::new("bytecode")
            .about("Display the Bytecode from a file")
            .arg(Arg::new("file").help("The file to compile").required(true)),
        ),
    )
    .get_matches();

  if run_command(&app).is_err() {
    std::process::exit(1)
  }
}

fn run_command(app: &clap::ArgMatches) -> Result<(), ()> {
  match app.subcommand() {
    Some(("run", args)) => {
      let filename = get_filename(args)?;
      let source = &read_file(filename)?;
      let bytecode = &compile(filename, source)?;

      run(filename, source, bytecode);
    }
    Some(("lint", args)) => {
      let filename = get_filename(args)?;
      let source = &read_file(filename)?;
      let ast = parse(filename, source)?;

      for diagnostic in bang::lint(source, &ast) {
        print::warning(filename, source, diagnostic);
      }
    }
    Some(("typecheck", args)) => {
      let filename = get_filename(args)?;
      let source = &read_file(filename)?;
      let ast = parse(filename, source)?;

      for diagnostic in bang::typecheck(source, &ast) {
        print::error(filename, source, &diagnostic);
      }
    }
    Some(("format", args)) => {
      let filename = get_filename(args)?;
      let source = &read_file(filename)?;
      let ast = parse(filename, source)?;
      let formatted_source = &bang::format(source, &ast);

      if args.is_present("dryrun") {
        return Ok(println!("{formatted_source}"));
      }

      if formatted_source != source && fs::write(filename, formatted_source).is_err() {
        print::error_message("Problem writing to file");
      }
    }
    Some(("print", args)) => match args.subcommand() {
      Some(("ast", args)) => {
        let filename = get_filename(args)?;
        let source = &read_file(filename)?;
        let ast = &parse(filename, source)?;

        print::ast(source, ast);
      }
      Some(("bytecode", args)) => {
        let filename = get_filename(args)?;
        let source = &read_file(filename)?;
        let bytecode = &compile(filename, source)?;

        print::chunk(bytecode);
      }
      _ => unreachable!(),
    },
    _ => repl(),
  };

  Ok(())
}

fn repl() {
  use rustyline::error::ReadlineError;

  println!("Bang! ({VERSION})");
  let mut rl = rustyline::Editor::<()>::new();
  let mut vm = bang::VM::new(&bang::StdContext);

  loop {
    let readline = rl.readline("> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(&line);

        let source = if line.starts_with("from")
          || line.starts_with("let")
          || line.starts_with("if")
          || line.starts_with("while")
        {
          line
        } else if line.is_empty() {
          continue;
        } else {
          format!("print({line})\n")
        };

        if let Ok(chunk) = compile("REPL", &source) {
          if let Err(error) = vm.run(&chunk) {
            print::runtime_error("REPL", &source, error);
          }
        }
      }
      Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
        break;
      }
      Err(err) => {
        print::error_message(&err.to_string());
        break;
      }
    }
  }
}
