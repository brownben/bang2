use crate::ast::{Expression, Statement};

pub fn ast(ast: &[Statement]) {
  println!("  ╭─[Abstract Syntax Tree]");
  for statement in ast {
    print_statement(statement, "  ├─ ".to_string(), "  │  ".to_string());
  }
  println!("──╯");
}

fn print_expression(expression: &Expression, prefix: String, prefix_raw: String) {
  match expression {
    Expression::Literal { value, .. } => println!("{}Literal ({})", prefix, value),
    Expression::Group { expression, .. } => {
      println!("{}Group", prefix);
      print_expression(&*expression, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
    }
    Expression::Unary {
      expression,
      operator,
      ..
    } => {
      println!("{}Unary ({})", prefix, operator);
      print_expression(&*expression, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
    }
    Expression::Binary {
      left,
      right,
      operator,
      ..
    } => {
      println!("{}Binary ({})", prefix, operator);
      print_expression(
        &*left,
        prefix_raw.clone() + "├─ ",
        prefix_raw.clone() + "│  ",
      );
      print_expression(&*right, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
    }
    Expression::Assignment {
      expression,
      variable_name,
      ..
    } => {
      println!("{}Assignment ({})", prefix, variable_name);
      print_expression(&*expression, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
    }
    Expression::Variable { variable_name, .. } => {
      println!("{}Variable ({})", prefix, variable_name);
    }
    Expression::Call {
      expression,
      arguments,
      ..
    } => {
      println!("{}Call", prefix);
      println!("{}├─ Expression", prefix_raw);
      print_expression(
        expression,
        prefix_raw.clone() + "│  ╰─ ",
        prefix_raw.clone() + "│     ",
      );
      println!("{}╰─ Arguments", prefix_raw);
      for arg in arguments {
        print_expression(
          arg,
          prefix_raw.clone() + "   ├─ ",
          prefix_raw.clone() + "   │  ",
        );
      }
    }
    Expression::Function {
      body, parameters, ..
    } => {
      println!(
        "{}Function ({})",
        prefix,
        parameters
          .iter()
          .map(|p| p.value.clone())
          .collect::<Vec<String>>()
          .join(", ")
      );
      print_statement(&*body, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
    }
  }
}

fn print_statement(statement: &Statement, prefix: String, prefix_raw: String) {
  match statement {
    Statement::Declaration {
      variable_name,
      expression,
      ..
    } => {
      println!("{}Declaration ({})", prefix, variable_name);
      if let Some(expression) = expression {
        print_expression(expression, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
      }
    }
    Statement::If {
      condition,
      then,
      otherwise,
      ..
    } => {
      println!("{}If", prefix);
      println!("{}├─ Condition", prefix_raw);
      print_expression(
        condition,
        prefix_raw.clone() + "│  ╰─ ",
        prefix_raw.clone() + "│     ",
      );
      println!("{}├─ Then", prefix_raw);
      print_statement(
        &*then,
        prefix_raw.clone() + "│  ╰─ ",
        prefix_raw.clone() + "│     ",
      );
      if let Some(ot) = otherwise {
        println!("{}╰─ Else", prefix_raw);
        print_statement(&*ot, prefix_raw.clone() + "   ╰─ ", prefix_raw + "      ");
      };
    }
    Statement::While {
      condition, body, ..
    } => {
      println!("{}While", prefix);
      println!("{}├─ Condition", prefix_raw);
      print_expression(
        condition,
        prefix_raw.clone() + "│  ╰─ ",
        prefix_raw.clone() + "│     ",
      );
      println!("{}╰─ Body", prefix_raw);
      print_statement(&*body, prefix_raw.clone() + "   ╰─ ", prefix_raw + "      ");
    }
    Statement::Return { expression, .. } => {
      println!("{}Return", prefix);
      if let Some(expression) = expression {
        print_expression(expression, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
      }
    }
    Statement::Block { body, .. } => {
      println!("{}Block", prefix);
      if let Some((last, statements)) = body.split_last() {
        for statement in statements {
          print_statement(
            statement,
            prefix_raw.clone() + "├─ ",
            prefix_raw.clone() + "│  ",
          );
        }
        print_statement(last, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
      }
    }
    Statement::Expression { expression, .. } => {
      println!("{}Expression", prefix);
      print_expression(expression, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
    }
  }
}
