use super::remove_carriage_returns;
use bang_language::ast::{
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
    Expr::List { items } => {
      println!("{}List", prefix_start);

      if let Some((last, items)) = items.split_last() {
        for item in items {
          print_expression(source, item, prefix_list_indent_start, prefix_list_indent);
        }
        print_expression(source, last, prefix_start_indent, prefix_blank_indent);
      }
    }
    Expr::Index { expression, index } => {
      println!("{}Index (expression, index)", prefix_start);

      print_expression(
        source,
        expression,
        prefix_list_indent_start,
        prefix_list_indent,
      );
      print_expression(source, index, prefix_start_indent, prefix_blank_indent);
    }
    Expr::IndexAssignment {
      expression,
      index,
      value,
      assignment_operator,
    } => {
      println!(
        "{}Index Assignment (expression[index] {} value)",
        prefix_start,
        assignment_operator
          .map(|operator| operator.to_string())
          .unwrap_or_else(|| "=".to_string())
      );

      print_expression(
        source,
        expression,
        prefix_list_indent_start,
        prefix_list_indent,
      );
      print_expression(source, index, prefix_list_indent_start, prefix_list_indent);
      print_expression(source, value, prefix_start_indent, prefix_blank_indent);
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
