use super::remove_carriage_returns;
use bang_syntax::ast::{
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
    Expr::Literal { value, .. } => println!("{prefix}Literal ({value})"),
    Expr::Group { expression, .. } => {
      println!("{prefix}Group");
      print_expression(source, expression, prefix_start, prefix_blank);
    }
    Expr::Unary {
      expression,
      operator,
    } => {
      println!("{prefix}Unary ({operator})");
      print_expression(source, expression, prefix_start, prefix_blank);
    }
    Expr::Binary {
      left,
      right,
      operator,
    } => {
      println!("{prefix}Binary ({operator})");
      print_expression(source, left, prefix_list_inline_start, prefix_list_inline);
      print_expression(source, right, prefix_start, prefix_blank);
    }
    Expr::Assignment {
      expression,
      identifier,
    } => {
      println!("{prefix}Assignment ({identifier})");
      print_expression(source, expression, prefix_start, prefix_blank);
    }
    Expr::Variable { name, .. } => {
      println!("{prefix}Variable ({name})");
    }
    Expr::Call {
      expression,
      arguments,
      ..
    } => {
      print_expression(source, expression, prefix, prefix_blank);
      println!("{prefix_start}Call");

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
        println!("{prefix}Function {name}({params})");
      } else {
        println!("{prefix}Function ({params})");
      }
      print_statement(source, body, prefix_start, prefix_blank);
    }
    Expr::Comment { expression, text } => {
      print_expression(source, expression, prefix, prefix_raw);
      println!(
        "{}Comment ({})",
        prefix_start,
        remove_carriage_returns(text)
      );
    }
    Expr::List { items } => {
      println!("{prefix_start}List");

      if let Some((last, items)) = items.split_last() {
        for item in items {
          print_expression(source, item, prefix_list_indent_start, prefix_list_indent);
        }
        print_expression(source, last, prefix_start_indent, prefix_blank_indent);
      }
    }
    Expr::Index { expression, index } => {
      println!("{prefix_start}Index (expression, index)");

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
    Expr::FormatString {
      strings,
      expressions,
    } => {
      println!("{prefix}Format String");

      println!("{prefix_list_inline_start}'{}'", strings[0]);
      for (index, item) in expressions.iter().enumerate() {
        print_expression(source, item, prefix_list_inline_start, prefix_list_inline);

        if index == expressions.len() - 1 {
          println!("{prefix_start}'{}'", strings[index + 1]);
        } else {
          println!("{prefix_list_inline_start}'{}'", strings[index + 1]);
        }
      }
    }
    Expr::ModuleAccess { module, item } => {
      println!("{prefix}Module Access ({module}::{item})");
    }
    Expr::Dictionary { items } => {
      println!("{prefix_start}Dictionary");

      for (key, value) in items {
        print_expression(source, key, prefix_list_indent_start, prefix_list_indent);
        print_expression(
          source,
          value,
          &format!("{prefix_list_indent}╰─ "),
          &format!("{prefix_raw}   │     "),
        );
      }
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
      println!("{prefix}Declaration ({identifier})");
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
      println!("{prefix}If");
      println!("{prefix_raw}├─ Condition");
      print_expression(source, condition, prefix_list_start, prefix_list);
      println!("{prefix_raw}├─ Then");
      print_statement(source, then, prefix_list_start, prefix_list);
      if let Some(ot) = otherwise {
        println!("{prefix_raw}╰─ Else");
        print_statement(source, ot, prefix_indetented_start, prefix_indetented);
      };
    }
    Stmt::Import { module, items, .. } => {
      println!("{prefix}From '{module}' Import");

      for item in items {
        if let Some(alias) = item.alias {
          println!("{prefix_raw}├─ {} as {alias}", item.name);
        } else {
          println!("{prefix_raw}├─ {}", item.name);
        }
      }
    }
    Stmt::While {
      condition, body, ..
    } => {
      println!("{prefix}While");
      println!("{prefix_raw}├─ Condition");
      print_expression(source, condition, prefix_list_start, prefix_list);
      println!("{prefix_raw}╰─ Body");
      print_statement(source, body, prefix_indetented_start, prefix_indetented);
    }
    Stmt::Return { expression, .. } => {
      println!("{prefix}Return");
      if let Some(expression) = expression {
        print_expression(source, expression, prefix_start, prefix_blank);
      }
    }
    Stmt::Block { body, .. } => {
      println!("{prefix}Block");
      if let Some((last, statements)) = body.split_last() {
        for stmt in statements {
          print_statement(source, stmt, prefix_list_inline_start, prefix_list_inline);
        }
        print_statement(source, last, prefix_start, prefix_blank);
      }
    }
    Stmt::Expression { expression, .. } => {
      println!("{prefix}Expression");
      print_expression(source, expression, prefix_start, prefix_blank);
    }
    Stmt::Comment { text, .. } => {
      println!("{prefix}Comment ({})", remove_carriage_returns(text));
    }
  }
}
