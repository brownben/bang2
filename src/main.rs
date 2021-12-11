#![warn(clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

mod ast;
mod chunk;
mod compiler;
mod error;
mod parser;
mod scanner;
mod token;
mod value;
mod vm;

use std::fs;

fn red(string: &str) -> String {
  format!("\x1b[0;31m{}\x1b[0m", string)
}

fn print_code_frame(file: &str, source: &str, line_number: usize) {
  eprintln!("    ╭─[{}]", file);
  if line_number > 1 {
    eprintln!("    ·");
  } else {
    eprintln!("    │");
  }
  for i in (line_number - 2)..=line_number {
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

  eprintln!("{} {}", red("Error:"), diagnostic.message);
  eprintln!("{}", diagnostic.label);
  print_code_frame(file, source, token.line as usize);
}

fn print_runtime_error(file: &str, source: &str, error: &error::RuntimeError) {
  eprintln!("{} {}", red("Error:"), error.message);
  print_code_frame(file, source, error.line_number as usize);
}

fn compile(source: &str) -> Result<chunk::Chunk, error::CompileError> {
  let ast = parser::parse(source)?;

  compiler::compile(ast)
}

fn main() {
  let filename = "./fib.bang";
  if let Ok(file) = fs::read_to_string(filename) {
    match compile(&file) {
      Ok(chunk) => match vm::run(chunk) {
        Ok(_) => {}
        Err(error) => print_runtime_error(filename, &file, &error),
      },
      Err(details) => print_compile_error(filename, &file, &details),
    }
  } else {
    println!("Problem reading file '{}'", filename);
  }
}
