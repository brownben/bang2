use crate::error::{get_message, CompileError, RuntimeError};
use crate::linter::RuleResult;

use ansi_term::Colour::{Red, Yellow};
use ansi_term::Style;

pub fn error(message: &str) {
  eprintln!("{} {}", Red.bold().paint("Error:"), message);
}

fn code_frame(file: &str, source: &str, line_number: usize) {
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

pub fn compile_error(file: &str, source: &str, error: &CompileError) {
  let CompileError { token, error } = error;
  let chars = source.to_string().chars().collect::<Vec<char>>();
  let diagnostic = get_message(&chars, error, token);

  eprintln!(
    "{} {}",
    Red.bold().paint("Compile Error:"),
    Style::new().bold().paint(diagnostic.message)
  );
  code_frame(file, source, token.line as usize);

  if let Some(note) = diagnostic.note {
    eprintln!("{}", note);
  }
}

pub fn runtime_error(file: &str, source: &str, error: &RuntimeError) {
  eprintln!(
    "{} {}",
    Red.bold().paint("Runtime Error:"),
    Style::new().bold().paint(&error.message)
  );

  for line_number in &error.line_numbers {
    if *line_number > 0 {
      code_frame(file, source, *line_number as usize);
    }
  }
}

pub fn lint_warning(file: &str, source: &str, result: &RuleResult) {
  eprintln!(
    "{} {}",
    Yellow.bold().paint("Warning:"),
    Style::new().bold().paint(result.name)
  );
  eprintln!("{}\n", result.message);

  for token in &result.issues {
    code_frame(file, source, token.line as usize);
  }
}
