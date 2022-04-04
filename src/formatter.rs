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

  fn write_group(
    &self,
    expression: &Expr,
    indentation: usize,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(f, "(")?;

    if expression.get_start().line == expression.get_end().line {
      self.fmt_expression(expression, indentation + 1, f)?;
      write!(f, ")")?;
    } else {
      write!(f, "\n{}", INDENTATION.repeat(indentation + 1))?;
      self.fmt_expression(expression, indentation + 1, f)?;
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

  fn write_list<Item>(
    &self,
    items: &[Item],
    get_line: impl Fn(&Item) -> LineNumber,
    write_item: &mut dyn FnMut(&mut std::fmt::Formatter, &Item, usize) -> std::fmt::Result,
    start: &Token,
    indentation: usize,
    padded: bool,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    let lines: Vec<LineNumber> = items.iter().map(get_line).collect();
    let all_same_line = lines.iter().all(|line| line == &lines[0]);
    let all_same_line_as_bracket = all_same_line && lines.contains(&start.line);

    if all_same_line_as_bracket || items.is_empty() {
      if padded {
        write!(f, " ")?;
      }
      for (i, item) in items.iter().enumerate() {
        write_item(f, item, indentation)?;
        if i < items.len() - 1 {
          write!(f, ", ")?;
        }
      }
      if padded {
        write!(f, " ")?;
      }
    } else if all_same_line {
      write!(f, "\n{}", INDENTATION.repeat(indentation + 1))?;
      for item in items {
        write_item(f, item, indentation + 1)?;
        write!(f, ", ")?;
      }
      writeln!(f)?;
    } else {
      for arg in items {
        write!(f, "\n{}", INDENTATION.repeat(indentation + 1))?;
        write_item(f, arg, indentation + 1)?;
        write!(f, ",")?;
      }
      write!(f, "\n{}", INDENTATION.repeat(indentation))?;
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
        self.fmt_expression(expression, indentation, f)?;

        write!(f, "(")?;
        self.write_list(
          arguments,
          |arg| arg.get_start().line,
          &mut |f, arg, i| self.fmt_expression(arg, i, f),
          token,
          indentation,
          false,
          f,
        )?;
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
        self.write_list(
          parameters,
          |param| param.line,
          &mut |f, parameter, _| write!(f, "{}", self.value(parameter)),
          token,
          indentation,
          false,
          f,
        )?;

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
        self.write_group( expression, indentation, f)?;
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
        write!(f, "if ")?;
        self.write_group(condition, indentation, f)?;
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
        self.write_list(
          items,
          |item| item.line,
          &mut |f, item, _| write!(f, "{}", self.value(item)),
          token,
          indentation,
          true,
          f,
        )?;
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
        write!(f, "while ")?;
        self.write_group(condition, indentation, f)?;
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
