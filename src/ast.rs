use crate::token::Token;
use crate::value::Value;

#[cfg(feature = "debug-ast")]
use crate::token::TokenType;

#[derive(Debug, Clone)]
pub struct Parameter {
  pub identifier: Token,
  pub value: String,
}

#[derive(Debug, Clone)]
pub enum Expression {
  Literal {
    value: Value,
    token: Token,
  },
  Group {
    expression: Box<Expression>,
  },
  Unary {
    operator: Token,
    expression: Box<Expression>,
  },
  Binary {
    left: Box<Expression>,
    operator: Token,
    right: Box<Expression>,
  },
  Assignment {
    identifier: Token,
    variable_name: String,
    expression: Box<Expression>,
  },
  Variable {
    variable_name: String,
    identifier: Token,
  },
  Call {
    expression: Box<Expression>,
    token: Token,
    arguments: Vec<Expression>,
  },
}

#[derive(Debug, Clone)]
pub enum Statement {
  Declaration {
    token: Token,
    identifier: Token,
    variable_name: String,
    expression: Option<Expression>,
  },
  If {
    if_token: Token,
    else_token: Option<Token>,
    condition: Expression,
    then: Box<Statement>,
    otherwise: Option<Box<Statement>>,
  },
  While {
    token: Token,
    condition: Expression,
    body: Box<Statement>,
  },
  Print {
    token: Token,
    expression: Expression,
  },
  Block {
    body: Vec<Statement>,
  },
  Expression {
    expression: Expression,
  },
  Function {
    name: String,
    token: Token,
    identifier: Token,
    parameters: Vec<Parameter>,
    body: Box<Statement>,
  },
}

#[cfg(feature = "debug-ast")]
pub fn print_ast(ast: &Vec<Statement>) {
  println!("  ╭─[Abstract Syntax Tree]");
  for statement in ast {
    print_statement(statement, "  ├─ ".to_string(), "  │  ".to_string());
  }
  println!("──╯");
}

#[cfg(feature = "debug-ast")]
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
      print!("{}Unary (", prefix);
      print_operator(operator);
      println!(")");
      print_expression(&*expression, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
    }
    Expression::Binary {
      left,
      right,
      operator,
      ..
    } => {
      print!("{}Binary (", prefix);
      print_operator(operator);
      println!(")");
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
      println!("{}Variable ({})", prefix, variable_name)
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
          &arg,
          prefix_raw.clone() + "   ├─ ",
          prefix_raw.clone() + "   │  ",
        );
      }
    }
  }
}

#[cfg(feature = "debug-ast")]
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
    Statement::Print { expression, .. } => {
      println!("{}Print", prefix);
      print_expression(expression, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
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
        print_statement(last, prefix_raw.clone() + "╰─ ", prefix_raw.clone() + "   ");
      }
    }
    Statement::Expression { expression, .. } => {
      println!("{}Expression", prefix);
      print_expression(expression, prefix_raw.clone() + "╰─ ", prefix_raw + "   ");
    }
    Statement::Function {
      name,
      body,
      parameters,
      ..
    } => {
      println!(
        "{}Function ({}) <{}>",
        prefix,
        name,
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

#[cfg(feature = "debug-ast")]
fn print_operator(operator: &Token) {
  match operator.token_type {
    TokenType::Minus => print!("-"),
    TokenType::Plus => print!("+"),
    TokenType::Slash => print!("/"),
    TokenType::Star => print!("*"),
    TokenType::Bang => print!("!"),
    TokenType::And => print!("and"),
    TokenType::Or => print!("or"),
    TokenType::QuestionQuestion => print!("??"),
    TokenType::BangEqual => print!("!="),
    TokenType::EqualEqual => print!("=="),
    TokenType::Greater => print!(">"),
    TokenType::GreaterEqual => print!(">="),
    TokenType::Less => print!("<"),
    TokenType::LessEqual => print!("<="),
    _ => print!(""),
  }
}
