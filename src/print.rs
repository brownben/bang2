use crate::{
  diagnostic::Diagnostic,
  tokens::{LineNumber, Token, TokenType},
};

mod format {
  pub fn red(text: &str) -> String {
    format!("\u{001b}[31m{}\u{001b}[0m", text)
  }

  pub fn yellow(text: &str) -> String {
    format!("\u{001b}[33m{}\u{001b}[0m", text)
  }

  pub fn bold(text: &str) -> String {
    format!("\u{001b}[1m{}\u{001b}[0m", text)
  }
}

pub fn tokens(tokens: &[Token]) {
  let mut line = 0;

  println!("     ╭─[Tokens]");
  for token in tokens {
    if token.line == line {
      print!("     │ ");
    } else {
      print!("{:>4} │ ", token.line);
      line = token.line;
    }

    let value = if token.ttype == TokenType::EndOfLine {
      ""
    } else {
      token.value
    };
    println!("{:?} ({})", token.ttype, value);
  }
  println!("─────╯");
}

pub use ast::print as ast;
mod ast {
  use crate::ast::{Expr, Stmt};

  pub fn print(ast: &[Stmt]) {
    println!("  ╭─[Abstract Syntax Tree]");
    for statement in ast {
      print_statement(statement, "  ├─ ", "  │  ");
    }
    println!("──╯");
  }

  fn print_expression(expression: &Expr, prefix: &str, prefix_raw: &str) {
    let prefix_list_start = &format!("{prefix_raw}│  ╰─ ");
    let prefix_list = &format!("{prefix_raw}│     ");
    let prefix_start = &format!("{prefix_raw}╰─ ");
    let prefix_blank = &format!("{prefix_raw}   ");
    let prefix_list_inline_start = &format!("{prefix_raw}├─ ");
    let prefix_list_inline = &format!("{prefix_raw}│  ");
    let prefix_list_indent_start = &format!("{prefix_raw}   ├─ ");
    let prefix_list_indent = &format!("{prefix_raw}   │  ");

    match expression {
      Expr::Literal { token, .. } => println!("{}Literal ({})", prefix, token.value),
      Expr::Group { expression } => {
        println!("{}Group", prefix);
        print_expression(&*expression, prefix_start, prefix_blank);
      }
      Expr::Unary {
        expression,
        operator,
      } => {
        println!("{}Unary ({})", prefix, operator.value);
        print_expression(&*expression, prefix_start, prefix_blank);
      }
      Expr::Binary {
        left,
        right,
        operator: token,
      } => {
        println!("{}Binary ({})", prefix, token.value);
        print_expression(&*left, prefix_list_inline_start, prefix_list_inline);
        print_expression(&*right, prefix_start, prefix_blank);
      }
      Expr::Assignment {
        expression,
        identifier,
      } => {
        println!("{}Assignment ({})", prefix, identifier.value);
        print_expression(&*expression, prefix_start, prefix_blank);
      }
      Expr::Variable { token, .. } => {
        println!("{}Variable ({})", prefix, token.value);
      }
      Expr::Call {
        expression,
        arguments,
        ..
      } => {
        println!("{}Call", prefix);
        println!("{}├─ Expression", prefix_raw);
        print_expression(expression, prefix_list_start, prefix_list);
        println!("{}╰─ Arguments", prefix_raw);
        for arg in arguments {
          print_expression(arg, prefix_list_indent_start, prefix_list_indent);
        }
      }
      Expr::Function {
        body, parameters, ..
      } => {
        let params: Vec<String> = parameters.iter().map(|p| p.value.to_string()).collect();
        println!("{}Function ({})", prefix, params.join(", "));
        print_statement(&*body, prefix_start, prefix_blank);
      }
    }
  }

  fn print_statement(statement: &Stmt, prefix: &str, prefix_raw: &str) {
    let prefix_indetented_start = &format!("{prefix_raw}   ╰─ ");
    let prefix_indetented = &format!("{prefix_raw}      ");
    let prefix_list_start = &format!("{prefix_raw}│  ╰─ ");
    let prefix_list = &format!("{prefix_raw}│     ");
    let prefix_start = &format!("{prefix_raw}╰─ ");
    let prefix_blank = &format!("{prefix_raw}   ");
    let prefix_list_inline_start = &format!("{prefix_raw}├─ ");
    let prefix_list_inline = &format!("{prefix_raw}│  ");

    match statement {
      Stmt::Declaration {
        identifier,
        expression,
        ..
      } => {
        println!("{}Declaration ({})", prefix, identifier.value);
        if let Some(expression) = expression {
          print_expression(expression, prefix_start, prefix_blank);
        }
      }
      Stmt::If {
        condition,
        then,
        otherwise,
        ..
      } => {
        println!("{}If", prefix);
        println!("{}├─ Condition", prefix_raw);
        print_expression(condition, prefix_list_start, prefix_list);
        println!("{}├─ Then", prefix_raw);
        print_statement(&*then, prefix_list_start, prefix_list);
        if let Some(ot) = otherwise {
          println!("{}╰─ Else", prefix_raw);
          print_statement(&*ot, prefix_indetented_start, prefix_indetented);
        };
      }
      Stmt::While {
        condition, body, ..
      } => {
        println!("{}While", prefix);
        println!("{}├─ Condition", prefix_raw);
        print_expression(condition, prefix_list_start, prefix_list);
        println!("{}╰─ Body", prefix_raw);
        print_statement(&*body, prefix_indetented_start, prefix_indetented);
      }
      Stmt::Return { expression, .. } => {
        println!("{}Return", prefix);
        if let Some(expression) = expression {
          print_expression(expression, prefix_start, prefix_blank);
        }
      }
      Stmt::Block { body, .. } => {
        println!("{}Block", prefix);
        if let Some((last, statements)) = body.split_last() {
          for stmt in statements {
            print_statement(stmt, prefix_list_inline_start, prefix_list_inline);
          }
          print_statement(last, prefix_start, prefix_blank);
        }
      }
      Stmt::Expression { expression, .. } => {
        println!("{}Expression", prefix);
        print_expression(expression, prefix_start, prefix_blank);
      }
    }
  }
}

pub use chunk::print as chunk;
mod chunk {
  use crate::{
    chunk::{Chunk, OpCode},
    value::Value,
  };

  pub fn print(chunk: &Chunk, name: &str) {
    disassemble_chunk(chunk, name);
    for constant in &chunk.constants {
      if let Value::Function(function) = constant {
        print(&function.chunk, "function");
      }
    }
  }

  fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("          ╭─[Bytecode:{}]", name);

    let mut position: usize = 0;
    let mut last_line_number = 0;

    while position < chunk.code.len() {
      let line_number = chunk.get_line_number(position);
      if line_number == last_line_number {
        print!("     {:0>4} │ ", position);
      } else {
        print!("{:<4} {:0>4} │ ", line_number, position);
        last_line_number = line_number;
      }

      position = disassemble_instruction(chunk, position);
    }
    println!("──────────╯");
  }

  fn disassemble_instruction(chunk: &Chunk, position: usize) -> usize {
    let instruction = chunk.get(position);

    match instruction {
      OpCode::Constant => constant_instruction("Constant", chunk, position),
      OpCode::ConstantLong => constant_long_instruction("Constant Long", chunk, position),
      OpCode::Null => simple_instruction("Null", position),
      OpCode::True => simple_instruction("True", position),
      OpCode::False => simple_instruction("False", position),
      OpCode::Add => simple_instruction("Add", position),
      OpCode::Subtract => simple_instruction("Subtract", position),
      OpCode::Multiply => simple_instruction("Multiply", position),
      OpCode::Divide => simple_instruction("Divide", position),
      OpCode::Negate => simple_instruction("Negate", position),
      OpCode::Not => simple_instruction("Not", position),
      OpCode::Equal => simple_instruction("Equal", position),
      OpCode::Greater => simple_instruction("Greater", position),
      OpCode::Less => simple_instruction("Less", position),
      OpCode::Pop => simple_instruction("Pop", position),
      OpCode::Return => simple_instruction("Return", position),
      OpCode::DefineGlobal => constant_instruction("Define Global", chunk, position),
      OpCode::GetGlobal => constant_instruction("Get Global", chunk, position),
      OpCode::SetGlobal => constant_instruction("Set Global", chunk, position),
      OpCode::Jump => jump_instruction("Jump", 1, chunk, position),
      OpCode::JumpIfFalse => jump_instruction("Jump If False", 1, chunk, position),
      OpCode::JumpIfNull => jump_instruction("Jump If Null", 1, chunk, position),
      OpCode::Loop => jump_instruction("Loop", -1, chunk, position),
      OpCode::GetLocal => byte_instruction("Get Local", chunk, position),
      OpCode::SetLocal => byte_instruction("Set Local", chunk, position),
      OpCode::Call => byte_instruction("Call", chunk, position),
      OpCode::Unknown => simple_instruction("Unknown OpCode", position),
    }
  }

  fn simple_instruction(name: &str, position: usize) -> usize {
    println!("{}", name);
    position + 1
  }

  fn constant_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
    let constant_location = chunk.get_value(position + 1);
    let constant = chunk.get_constant(constant_location as usize);

    println!("{} {} ({})", name, constant, constant_location);

    position + 2
  }

  fn constant_long_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
    let constant_location = chunk.get_long_value(position + 1);
    let constant = chunk.get_constant(constant_location as usize);

    println!("{} '{}' ({})", name, constant, constant_location);
    position + 3
  }

  fn byte_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
    let value = chunk.get_value(position + 1);

    println!("{} {}", name, value);
    position + 2
  }

  fn jump_instruction(name: &str, direction: i8, chunk: &Chunk, position: usize) -> usize {
    let jump = chunk.get_long_value(position + 1);

    println!("{} {}", name, i32::from(jump) * i32::from(direction));
    position + 3
  }
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

pub fn error(filename: &str, source: &str, diagnostic: Diagnostic) {
  eprintln!(
    "{} {}\n{}\n",
    format::bold(&format::red("Error:")),
    format::bold(&diagnostic.title),
    diagnostic.message
  );

  for line_number in diagnostic.lines {
    code_frame(filename, source, line_number);
  }
}

pub fn warning(filename: &str, source: &str, diagnostic: Diagnostic) {
  eprintln!(
    "{} {}\n{}\n",
    format::bold(&format::yellow("Warning:")),
    format::bold(&diagnostic.title),
    diagnostic.message
  );

  for line_number in diagnostic.lines {
    code_frame(filename, source, line_number);
  }
}
