use crate::{
  ast::{Expr, GetPosition, Stmt},
  tokens::{LineNumber, Token, TokenType},
  Value,
};

const INDENTATION: &str = "  ";

struct Formatter<'source> {
  source: &'source [u8],
  ast: &'source [Stmt<'source>],
}

impl<'source> Formatter<'source> {
  fn new(source: &'source str, ast: &'source [Stmt<'source>]) -> Self {
    Self {
      source: source.as_bytes(),
      ast,
    }
  }

  fn value(&self, token: &Token) -> &str {
    token.get_value(self.source)
  }

  fn write_condition(
    &self,
    keyword: &str,
    condition: &Expr,
    indentation: usize,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(f, "{} (", keyword)?;

    if condition.get_start().line == condition.get_end().line {
      self.fmt_expression(condition, indentation + 1, f)?;
      write!(f, ")")?;
    } else {
      write!(f, "\n{}", INDENTATION.repeat(indentation + 1))?;
      self.fmt_expression(condition, indentation + 1, f)?;
      write!(f, "\n{})", INDENTATION.repeat(indentation))?;
    }

    Ok(())
  }

  fn write_statement_inline(
    &self,
    statement: &Stmt,
    indentation: usize,
    trailing_newline: bool,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    if let Stmt::Block { body, .. } = statement {
      if body.len() > 1 {
        writeln!(f)?;
        self.fmt_statement(statement, indentation, true, f)?;
      } else {
        write!(f, " ")?;
        self.fmt_statement(&body[0], indentation, false, f)?;
        if trailing_newline {
          writeln!(f)?;
        }
      }
    } else {
      write!(f, " ")?;
      self.fmt_statement(statement, indentation, false, f)?;
      if trailing_newline {
        writeln!(f)?;
      }
    }

    Ok(())
  }

  fn fmt_expression(
    &self,
    expr: &Expr,
    indentation: usize,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    match expr {
      Expr::Assignment {
        identifier,
        expression,
        ..
      } => {
        if let Expr::Binary { operator, left, right, .. } = &**expression
          && let Expr::Variable { token } = &**left
          && self.value(token) == self.value(identifier)
          && operator.ttype.get_corresponding_assignment_operator().is_some()
        {
          let operator = operator.ttype.get_corresponding_assignment_operator().unwrap();
          write!(f, "{} {} ", self.value(identifier), operator)?;
          self.fmt_expression(right, indentation, f)?;
        } else {
          write!(f, "{} = ", self.value(identifier))?;
          self.fmt_expression(expression, indentation, f)?;
        }
      }
      Expr::Binary {
        left,
        right,
        operator,
        ..
      } => {
        self.fmt_expression(left, indentation, f)?;
        write!(f, " {} ", operator.ttype)?;
        self.fmt_expression(right, indentation, f)?;
      }
      Expr::Call {
        expression,
        arguments,
        token,
        ..
      } => {
        let argument_lines: Vec<LineNumber> =
          arguments.iter().map(|arg| arg.get_start().line).collect();
        let all_same_line = argument_lines.iter().all(|line| line == &argument_lines[0]);
        let all_same_line_as_bracket = all_same_line && argument_lines.contains(&token.line);

        self.fmt_expression(expression, indentation, f)?;

        write!(f, "(")?;

        if all_same_line_as_bracket || arguments.is_empty() {
          for (i, arg) in arguments.iter().enumerate() {
            self.fmt_expression(arg, indentation, f)?;
            if i < arguments.len() - 1 {
              write!(f, ", ")?;
            }
          }
        } else if all_same_line {
          write!(f, "\n{}", INDENTATION.repeat(indentation + 1))?;
          for arg in arguments {
            self.fmt_expression(arg, indentation + 1, f)?;
            write!(f, ", ")?;
          }
          writeln!(f)?;
        } else {
          for arg in arguments {
            write!(f, "\n{}", INDENTATION.repeat(indentation + 1))?;
            self.fmt_expression(arg, indentation + 1, f)?;
            write!(f, ",")?;
          }
          write!(f, "\n{}", INDENTATION.repeat(indentation))?;
        }

        write!(f, ")")?;
      }
      Expr::Comment {
        expression, token, ..
      } => {
        let message = self.value(token).replace("//", "");

        self.fmt_expression(expression, indentation, f)?;
        write!(f, " // {}", message.trim())?;
      }
      Expr::Function {
        token,
        parameters,
        body,
        ..
      } => {
        write!(f, "(")?;

        let lines: Vec<LineNumber> = parameters.iter().map(|parameter| parameter.line).collect();
        let all_same_line = lines.iter().all(|line| line == &lines[0]);
        let all_same_line_as_bracket = all_same_line && lines.contains(&token.line);

        if all_same_line_as_bracket || parameters.is_empty() {
          for (i, parameter) in parameters.iter().enumerate() {
            write!(f, "{}",self.value(parameter))?;
            if i < parameters.len() - 1 {
              write!(f, ", ")?;
            }
          }
        } else if all_same_line {
          write!(f, "\n{}", INDENTATION.repeat(indentation + 1))?;
          for parameter in parameters {
            write!(f, "{}, ", self.value(parameter))?;
          }
          writeln!(f)?;
        } else {
          for parameter in parameters {
            write!(f, "\n{}{},", INDENTATION.repeat(indentation + 1), self.value(parameter))?;
          }
          write!(f, "\n{}", INDENTATION.repeat(indentation))?;
        }

        if let Stmt::Return {
          expression: Some(expression),
          ..
        } = &**body
        {
          write!(f, ") => ")?;
          self.fmt_expression(expression, indentation, f)?;
        } else {
          writeln!(f, ") ->")?;
          self.fmt_statement(body, indentation, false, f)?;
        }
      }
      Expr::Group { expression, .. } => {
        write!(f, "(")?;
        self.fmt_expression(expression, indentation, f)?;
        write!(f, ")")?;
      }
      Expr::Literal { token, value, .. } => {
        match token.ttype {
          TokenType::String => write!(f, "'{}'", value)?,
          TokenType::Number => {
            if str::contains(value, "_") {
              write!(f, "{}", value)?;
            } else {
              write!(f, "{}", Value::parse_number(value))?;
            }
          }
          TokenType::True => write!(f, "true")?,
          TokenType::False => write!(f, "false")?,
          TokenType::Null => write!(f, "null")?,
          _ => unreachable!(),
        };
      }
      Expr::Unary {
        operator,
        expression,
        ..
      } => {
        write!(f, "{}", operator.ttype)?;
        self.fmt_expression(expression, indentation, f)?;
      }
      Expr::Variable { token, .. } => {
        write!(f, "{}", self.value(token))?;
      }
    }

    Ok(())
  }

  fn fmt_statement(
    &self,
    stmt: &Stmt,
    indentation: usize,
    new_line: bool,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    let mut ending_new_line = new_line;

    match stmt {
      Stmt::Block { body, .. } => {
        if let Some((last, body)) = body.split_last() {
          for stmt in body {
            write!(f, "{}", INDENTATION.repeat(indentation + 1))?;
            self.fmt_statement(stmt, indentation + 1, true, f)?;
          }
          write!(f, "{}", INDENTATION.repeat(indentation + 1))?;
          self.fmt_statement(last, indentation + 1, false, f)?;
        }
        ending_new_line = false;
      }
      Stmt::Comment { token, .. } => {
        let message = self.value(token).replace("//", "");
        write!(f, "// {}", message.trim())?;
      }
      Stmt::Declaration {
        identifier,
        expression,
        ..
      } => {
        write!(f, "let {} = ", self.value(identifier))?;
        if let Some(expression) = expression {
          self.fmt_expression(expression, indentation, f)?;
        }
      }
      Stmt::Expression { expression, .. } => {
        self.fmt_expression(expression, indentation, f)?;
      }
      Stmt::If {
        condition,
        then,
        otherwise,
        ..
      } => {
        self.write_condition("if", condition, indentation, f)?;
        self.write_statement_inline(then, indentation, false, f)?;

        if let Some(otherwise) = otherwise {
          write!(f, "\n{}else", INDENTATION.repeat(indentation))?;
          self.write_statement_inline(otherwise, indentation, false, f)?;
        }
      }
      Stmt::Import {
        token,
        module,
        items,
        ..
      } => {
        write!(f, "from {} import {{", self.value(module))?;

        let lines: Vec<LineNumber> = items.iter().map(|item| item.line).collect();
        let all_same_line = lines.iter().all(|line| line == &lines[0]);
        let all_same_line_as_bracket = all_same_line && lines.contains(&token.line);

        if all_same_line_as_bracket || items.is_empty() {
          write!(f, " ")?;
          for (i, item) in items.iter().enumerate() {
            write!(f, "{}", self.value(item))?;
            if i < items.len() - 1 {
              write!(f, ", ")?;
            }
          }
          write!(f, " ")?;
        } else if all_same_line {
          write!(f, "\n{}", INDENTATION.repeat(indentation + 1))?;
          for item in items {
            write!(f, "{}, ", self.value(item))?;
          }
          writeln!(f)?;
        } else {
          for item in items {
            write!(
              f,
              "\n{}{},",
              INDENTATION.repeat(indentation + 1),
              self.value(item)
            )?;
          }
          write!(f, "\n{}", INDENTATION.repeat(indentation))?;
        }
        write!(f, "}}")?;
      }
      Stmt::Return { expression, .. } => {
        write!(f, "return ")?;
        if let Some(expression) = expression {
          self.fmt_expression(expression, indentation, f)?;
        }
      }
      Stmt::While {
        condition, body, ..
      } => {
        self.write_condition("while", condition, indentation, f)?;
        self.write_statement_inline(&**body, indentation, false, f)?;
      }
    }

    if ending_new_line {
      writeln!(f)
    } else {
      Ok(())
    }
  }
}

impl std::fmt::Display for Formatter<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    if self.ast.is_empty() {
      return Ok(());
    }

    let mut prev = &self.ast[0];
    for stmt in self.ast {
      if prev.get_end().line + 1 < stmt.get_start().line {
        writeln!(f)?;
      }

      self.fmt_statement(stmt, 0, true, f)?;
      prev = stmt;
    }

    Ok(())
  }
}

pub fn format(source: &str, ast: &[Stmt]) -> String {
  Formatter::new(source, ast).to_string()
}
