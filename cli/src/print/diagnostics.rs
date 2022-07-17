use super::remove_carriage_returns;
use bang_interpreter::RuntimeError;
use bang_syntax::{Diagnostic, LineNumber};
use bang_tools::LintDiagnostic;

fn red(text: &str) -> String {
  format!("\u{001b}[31m{}\u{001b}[0m", text)
}

fn yellow(text: &str) -> String {
  format!("\u{001b}[33m{}\u{001b}[0m", text)
}

fn bold(text: &str) -> String {
  format!("\u{001b}[1m{}\u{001b}[0m", text)
}

fn code_frame(file: &str, source: &str, line_number: LineNumber) {
  eprintln!("    ╭─[{}]", file);
  if line_number > 2 {
    eprintln!("    ·");
  } else {
    eprintln!("    │");
  }

  let start = if line_number <= 2 { 0 } else { line_number - 2 };
  for i in start..=line_number {
    if let Some(line) = source.lines().nth(i as usize) {
      eprintln!("{:>3} │ {}", i + 1, line);
    }
  }
  if (line_number as usize) < (source.lines().count() - 1) {
    eprintln!("    ·");
  }
  eprintln!("────╯");
}

pub fn runtime_error(filename: &str, source: &str, error: RuntimeError) {
  eprintln!("{} {}\n", bold(&red("Error:")), bold(&error.message),);

  for line_number in error.lines {
    code_frame(filename, source, line_number);
  }
}

pub fn error(filename: &str, source: &str, diagnostic: &Diagnostic) {
  eprintln!(
    "{} {}\n{}\n",
    bold(&red("Error:")),
    bold(&diagnostic.title),
    remove_carriage_returns(&diagnostic.message)
  );

  code_frame(filename, source, diagnostic.line);
}

pub fn warning(filename: &str, source: &str, diagnostic: LintDiagnostic) {
  eprintln!(
    "{} {}\n{}\n",
    bold(&yellow("Warning:")),
    bold(&diagnostic.title),
    diagnostic.message
  );

  for line_number in diagnostic.lines {
    code_frame(filename, source, line_number);
  }
}
