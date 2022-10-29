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

pub fn error_message(message: &str) {
  eprintln!("{} {}", bold(&red("Error:")), bold(message),);
}

pub fn warning_message(message: &str) {
  eprintln!("{} {}", bold(&yellow("Warning:")), bold(message),);
}
