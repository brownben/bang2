mod ast;
mod builtin;
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

  eprintln!("{} {}", red("Error:"), diagnostic.message);
  print_code_frame(file, source, token.line as usize);

  if let Some(note) = diagnostic.note {
    eprintln!("{}", note);
  }
}

fn print_runtime_error(file: &str, source: &str, error: &error::RuntimeError) {
  eprintln!("{} {}", red("Error:"), error.message);

  for line_number in &error.line_numbers {
    if *line_number > 0 {
      print_code_frame(file, source, *line_number as usize);
    }
  }
}

fn compile(source: &str) -> Result<chunk::Chunk, error::CompileError> {
  let ast = parser::parse(source)?;

  compiler::compile(ast)
}

fn main() {
  let filename = "./test.bang";
  if let Ok(file) = fs::read_to_string(filename) {
    match compile(&file) {
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
