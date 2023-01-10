use bang_interpreter::errors;
use bang_syntax::LineNumber;

fn red(text: &str) -> String {
  format!("\u{001b}[31m{text}\u{001b}[0m")
}

fn yellow(text: &str) -> String {
  format!("\u{001b}[33m{text}\u{001b}[0m")
}

fn bold(text: &str) -> String {
  format!("\u{001b}[1m{text}\u{001b}[0m")
}

pub fn code_frame(file: &str, source: &str, line_number: LineNumber) {
  eprintln!("    ╭─[{file}]");
  if line_number > 2 {
    eprintln!("    ·");
  } else {
    eprintln!("    │");
  }

  let start = if line_number <= 2 { 0 } else { line_number - 2 };
  for i in start..=line_number {
    if let Some(line) = source.lines().nth(i as usize) {
      eprintln!("{:>3} │ {line}", i + 1);
    }
  }
  if (line_number as usize) < (source.lines().count() - 1) {
    eprintln!("    ·");
  }
  eprintln!("────╯");
}

pub fn stack_trace(filename: &str, source: &str, error: errors::Runtime) {
  error_message(&error.message);

  if error.stack[0].line != u16::MAX {
    code_frame(filename, source, error.stack[0].line);
  }

  for location in error.stack {
    match &location.kind {
      errors::StackTraceLocationKind::Root => {
        eprintln!("    at line {}", location.line);
      }
      errors::StackTraceLocationKind::Function(name) if name.is_empty() => {
        eprintln!("    in anonymous function at line {}", location.line);
      }
      errors::StackTraceLocationKind::Function(name) => {
        eprintln!("    in function '{name}' at line {}", location.line);
      }
      errors::StackTraceLocationKind::Builtin => {
        eprintln!("    in builtin function");
      }
    };
  }
}

pub fn error_message(message: &str) {
  eprintln!("{} {}", bold(&red("Error:")), bold(message),);
}

pub fn warning_message(message: &str) {
  eprintln!("{} {}", bold(&yellow("Warning:")), bold(message),);
}
