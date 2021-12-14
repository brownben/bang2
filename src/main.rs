mod ast;
mod builtin;
mod chunk;
mod compiler;
mod error;
mod linter;
mod parser;
mod scanner;
mod token;
mod value;
mod vm;

use crate::linter::Rule;

use ansi_term::Colour::{Red, Yellow};
use ansi_term::Style;
use std::fs;

fn print_code_frame(file: &str, source: &str, line_number: usize) {
  eprintln!("    ╭─[{}]", file);
  if line_number > 2 {
    eprintln!("    ·");
  } else {
    eprintln!("    │");
  }

  let before = if line_number >= 2 { 2 } else { 1 };
  for i in (line_number - before)..=line_number {
    if let Some(line) = source.lines().nth(i as usize) {
      eprintln!("{:>3} │ {}", i + 1, line);
    }
  }
  if line_number < source.lines().count() - 1 {
    eprintln!("    ·");
  }
  eprintln!("────╯");
}

fn print_compile_error(file: &str, source: &str, error: &error::CompileError) {
  let error::CompileError { token, error } = error;
  let chars = source.to_string().chars().collect::<Vec<char>>();
  let diagnostic = error::get_message(&chars, error, token);

  eprintln!(
    "{} {}",
    Red.bold().paint("Complie Error:"),
    Style::new().bold().paint(diagnostic.message)
  );
  print_code_frame(file, source, token.line as usize);

  if let Some(note) = diagnostic.note {
    eprintln!("{}", note);
  }
}

fn print_runtime_error(file: &str, source: &str, error: &error::RuntimeError) {
  eprintln!(
    "{} {}",
    Red.bold().paint("Runtime Error:"),
    Style::new().bold().paint(&error.message)
  );

  for line_number in &error.line_numbers {
    if *line_number > 0 {
      print_code_frame(file, source, *line_number as usize);
    }
  }
}

fn print_lint_warning(file: &str, source: &str, result: &linter::RuleResult) {
  eprintln!(
    "{} {}",
    Yellow.bold().paint("Warning:"),
    Style::new().bold().paint(result.name)
  );
  eprintln!("{}\n", result.message);

  for token in &result.issues {
    print_code_frame(file, source, token.line as usize);
  }
}

fn compile(file: &str, source: &str) -> Result<chunk::Chunk, error::CompileError> {
  let ast = parser::parse(source)?;

  if let Some(warning) = linter::NoNegativeZero::check(&ast) {
    print_lint_warning(file, source, &warning);
  }
  if let Some(warning) = linter::NoUnreachable::check(&ast) {
    print_lint_warning(file, source, &warning);
  }

  compiler::compile(ast)
}

fn main() {
  let filename = "./test.bang";
  if let Ok(file) = fs::read_to_string(filename) {
    match compile(&filename, &file) {
      Ok(chunk) => match vm::run(chunk) {
        Ok(_) => {}
        Err(error) => print_runtime_error(filename, &file, &error),
      },
      Err(details) => print_compile_error(filename, &file, &details),
    }
  } else {
    eprintln!("Problem reading file '{}'", filename);
  }
}
