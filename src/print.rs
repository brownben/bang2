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

fn remove_carriage_returns(value: &str) -> String {
  str::replace(value, "\r", "")
}

pub fn tokens(source: &str, tokens: &[Token]) {
  let source = source.as_bytes();
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
      token.get_value(source)
    };
    println!("{:?} ({})", token.ttype, remove_carriage_returns(value));
  }
  println!("─────╯");
}

pub use ast::print as ast;
mod ast {
  use super::remove_carriage_returns;
  use crate::ast::{
    expression::{Expr, Expression},
    statement::{Statement, Stmt},
  };

  pub fn print(source: &str, ast: &[Statement]) {
    let source = source.as_bytes();

    println!("  ╭─[Abstract Syntax Tree]");
    for statement in ast {
      print_statement(source, statement, "  ├─ ", "  │  ");
    }
    println!("──╯");
  }

  fn print_expression(source: &[u8], expression: &Expression, prefix: &str, prefix_raw: &str) {
    let prefix_start = &format!("{prefix_raw}╰─ ");
    let prefix_blank = &format!("{prefix_raw}   ");
    let prefix_start_indent = &format!("{prefix_raw}   ╰─ ");
    let prefix_blank_indent = &format!("{prefix_raw}      ");
    let prefix_list_inline_start = &format!("{prefix_raw}├─ ");
    let prefix_list_inline = &format!("{prefix_raw}│  ");
    let prefix_list_indent_start = &format!("{prefix_raw}   ├─ ");
    let prefix_list_indent = &format!("{prefix_raw}   │  ");

    match &expression.expr {
      Expr::Literal { value, .. } => println!("{}Literal ({})", prefix, value),
      Expr::Group { expression, .. } => {
        println!("{}Group", prefix);
        print_expression(source, &*expression, prefix_start, prefix_blank);
      }
      Expr::Unary {
        expression,
        operator,
      } => {
        println!("{}Unary ({})", prefix, operator);
        print_expression(source, &*expression, prefix_start, prefix_blank);
      }
      Expr::Binary {
        left,
        right,
        operator,
      } => {
        println!("{}Binary ({})", prefix, operator);
        print_expression(source, &*left, prefix_list_inline_start, prefix_list_inline);
        print_expression(source, &*right, prefix_start, prefix_blank);
      }
      Expr::Assignment {
        expression,
        identifier,
      } => {
        println!("{}Assignment ({})", prefix, identifier);
        print_expression(source, &*expression, prefix_start, prefix_blank);
      }
      Expr::Variable { name, .. } => {
        println!("{}Variable ({})", prefix, name);
      }
      Expr::Call {
        expression,
        arguments,
        ..
      } => {
        print_expression(source, &*expression, prefix, prefix_blank);
        println!("{}Call", prefix_start);

        if let Some((last, arguments)) = arguments.split_last() {
          for arg in arguments {
            print_expression(source, arg, prefix_list_indent_start, prefix_list_indent);
          }
          print_expression(source, last, prefix_start_indent, prefix_blank_indent);
        }
      }
      Expr::Function {
        body,
        parameters,
        name,
        ..
      } => {
        let params = parameters
          .iter()
          .map(|param| param.name)
          .collect::<Vec<_>>()
          .join(", ");

        if let Some(name) = name {
          println!("{}Function {}({})", prefix, name, params);
        } else {
          println!("{}Function ({})", prefix, params);
        }
        print_statement(source, &*body, prefix_start, prefix_blank);
      }
      Expr::Comment { expression, text } => {
        print_expression(source, &*expression, prefix, prefix_raw);
        println!(
          "{}Comment ({})",
          prefix_start,
          remove_carriage_returns(text)
        );
      }
    }
  }

  fn print_statement(source: &[u8], statement: &Statement, prefix: &str, prefix_raw: &str) {
    let prefix_indetented_start = &format!("{prefix_raw}   ╰─ ");
    let prefix_indetented = &format!("{prefix_raw}      ");
    let prefix_list_start = &format!("{prefix_raw}│  ╰─ ");
    let prefix_list = &format!("{prefix_raw}│     ");
    let prefix_start = &format!("{prefix_raw}╰─ ");
    let prefix_blank = &format!("{prefix_raw}   ");
    let prefix_list_inline_start = &format!("{prefix_raw}├─ ");
    let prefix_list_inline = &format!("{prefix_raw}│  ");

    match &statement.stmt {
      Stmt::Declaration {
        identifier,
        expression,
        ..
      } => {
        println!("{}Declaration ({})", prefix, identifier);
        if let Some(expression) = expression {
          print_expression(source, expression, prefix_start, prefix_blank);
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
        print_expression(source, condition, prefix_list_start, prefix_list);
        println!("{}├─ Then", prefix_raw);
        print_statement(source, &*then, prefix_list_start, prefix_list);
        if let Some(ot) = otherwise {
          println!("{}╰─ Else", prefix_raw);
          print_statement(source, &*ot, prefix_indetented_start, prefix_indetented);
        };
      }
      Stmt::Import { module, items, .. } => {
        println!("{}From '{}' Import", prefix, module);

        for item in items {
          if let Some(alias) = item.alias {
            println!("{}├─ {} as {}", prefix_raw, item.name, alias);
          } else {
            println!("{}├─ {}", prefix_raw, item.name);
          }
        }
      }
      Stmt::While {
        condition, body, ..
      } => {
        println!("{}While", prefix);
        println!("{}├─ Condition", prefix_raw);
        print_expression(source, condition, prefix_list_start, prefix_list);
        println!("{}╰─ Body", prefix_raw);
        print_statement(source, &*body, prefix_indetented_start, prefix_indetented);
      }
      Stmt::Return { expression, .. } => {
        println!("{}Return", prefix);
        if let Some(expression) = expression {
          print_expression(source, expression, prefix_start, prefix_blank);
        }
      }
      Stmt::Block { body, .. } => {
        println!("{}Block", prefix);
        if let Some((last, statements)) = body.split_last() {
          for stmt in statements {
            print_statement(source, stmt, prefix_list_inline_start, prefix_list_inline);
          }
          print_statement(source, last, prefix_start, prefix_blank);
        }
      }
      Stmt::Expression { expression, .. } => {
        println!("{}Expression", prefix);
        print_expression(source, expression, prefix_start, prefix_blank);
      }
      Stmt::Comment { text, .. } => {
        println!("{}Comment ({})", prefix, remove_carriage_returns(text));
      }
    }
  }
}

pub use chunk::print as chunk;
mod chunk {
  use crate::chunk::{Chunk, OpCode};

  pub fn print(chunk: &Chunk) {
    println!("          ╭─[Bytecode]");

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
    remove_carriage_returns(&diagnostic.message)
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
