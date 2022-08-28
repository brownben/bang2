use super::{bang, print};
use std::fs;

pub fn get_filename(args: &clap::ArgMatches) -> Result<&str, ()> {
  if let Some(filename) = args.value_of("file") {
    Ok(filename)
  } else {
    print::error_message("No file specified");
    Err(())
  }
}

pub fn read_file(filename: &str) -> Result<String, ()> {
  if let Ok(file) = fs::read_to_string(filename) {
    if file.is_empty() {
      print::warning_message("File is empty");
    }

    Ok(file)
  } else {
    print::error_message("Problem reading file");
    Err(())
  }
}

pub fn parse<'a>(filename: &str, source: &'a str) -> Result<bang::Ast<'a>, ()> {
  match bang::parse(source) {
    Ok(statements) => Ok(statements),
    Err(diagnostic) => {
      print::error(filename, source, &diagnostic);
      Err(())
    }
  }
}

pub fn compile(filename: &str, source: &str) -> Result<bang::Chunk, ()> {
  match bang::compile(source, &bang::StdContext) {
    Ok(chunk) => Ok(chunk),
    Err(diagnostic) => {
      print::error(filename, source, &diagnostic);
      Err(())
    }
  }
}

pub fn run(filename: &str, source: &str, chunk: &bang::Chunk) {
  match bang::VM::new(&bang::StdContext).run(chunk) {
    Ok(()) => {}
    Err(error) => print::runtime_error(filename, source, error),
  }
}
