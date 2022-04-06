use crate::{
  ast::{AssignmentOperator, Expr, Expression, LiteralType, Span, Statement, Stmt},
  parser::parse_number,
  tokens::LineNumber,
};

const INDENTATION: &str = "  ";

struct Formatter<'source> {
  source: &'source str,
  ast: &'source [Statement<'source>],
}

impl<'source> Formatter<'source> {
  fn new(source: &'source str, ast: &'source [Statement<'source>]) -> Self {
    Self { source, ast }
  }

  fn line(&self, span: &Span) -> LineNumber {
    span.get_line_number(self.source)
  }

  fn line_end(&self, span: &Span) -> LineNumber {
    span.get_line_number_end(self.source)
  }

  fn write_group(
    &self,
    expression: &Expression,
    indentation: usize,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(f, "(")?;
    if self.line(&expression.span) == self.line_end(&expression.span) {
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
    statement: &Statement,
    indentation: usize,
    trailing_newline: bool,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    if let Stmt::Block { body, .. } = &statement.stmt {
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
    start_line: LineNumber,
    indentation: usize,
    padded: bool,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    let lines: Vec<LineNumber> = items.iter().map(get_line).collect();
    let all_same_line = lines.iter().all(|line| line == &lines[0]);
    let all_same_line_as_bracket = all_same_line && lines.contains(&start_line);

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
    expression: &Expression,
    indentation: usize,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    let span = expression.span;

    match &expression.expr {
      Expr::Assignment {
        identifier,
        expression,
        ..
      } => {
        if let Expr::Binary { operator, left, right, .. } = &expression.expr
          && let Expr::Variable { name } = &left.expr
          && let Some(assignment_operator) = AssignmentOperator::from_binary(operator)
          && name == identifier
        {
          write!(f, "{identifier} {assignment_operator} ")?;
          self.fmt_expression(right, indentation, f)?;
        } else {
          write!(f, "{identifier} = ")?;
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
        write!(f, " {operator} ")?;
        self.fmt_expression(right, indentation, f)?;
      }
      Expr::Call {
        expression,
        arguments,
        ..
      } => {
        self.fmt_expression(expression, indentation, f)?;

        write!(f, "(")?;
        self.write_list(
          arguments,
          |arg| self.line(&arg.span),
          &mut |f, arg, i| self.fmt_expression(arg, i, f),
          self.line(&expression.span),
          indentation,
          false,
          f,
        )?;
        write!(f, ")")?;
      }
      Expr::Comment {
        expression, text, ..
      } => {
        let message = text.replacen("//", "", 1);

        self.fmt_expression(expression, indentation, f)?;
        write!(f, " // {}", message.trim())?;
      }
      Expr::Function {
        parameters,
        body,
        ..
      } => {
        write!(f, "(")?;
        self.write_list(
          parameters,
          |param| self.line(&param.span),
          &mut |f, parameter, _| write!(f, "{}", parameter.name),
          self.line(&span),
          indentation,
          false,
          f,
        )?;

        if let Stmt::Return {
          expression: Some(expression),
          ..
        } = &body.stmt
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
      Expr::Literal { type_, value, .. } => {
        match type_ {
          LiteralType::String => write!(f, "'{value}'")?,
          LiteralType::Number => {
            if str::contains(value, "_") {
              write!(f, "{value}")?;
            } else {
              write!(f, "{}", parse_number(value))?;
            }
          }
          LiteralType::True => write!(f, "true")?,
          LiteralType::False => write!(f, "false")?,
          LiteralType::Null => write!(f, "null")?,
        };
      }
      Expr::Unary {
        operator,
        expression,
        ..
      } => {
        write!(f, "{operator}")?;
        self.fmt_expression(expression, indentation, f)?;
      }
      Expr::Variable { name, .. } => {
        write!(f, "{name}")?;
      }
    }

    Ok(())
  }

  fn fmt_statement(
    &self,
    statement: &Statement,
    indentation: usize,
    new_line: bool,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    let mut ending_new_line = new_line;
    let span = statement.span;

    match &statement.stmt {
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
      Stmt::Comment { text, .. } => {
        let message = text.replacen("//", "", 1);
        write!(f, "// {}", message.trim())?;
      }
      Stmt::Declaration {
        identifier,
        expression,
        ..
      } => {
        write!(f, "let {identifier} = ")?;
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
      Stmt::Import { module, items, .. } => {
        write!(f, "from {module} import {{")?;
        self.write_list(
          items,
          |item| self.line(&item.span),
          &mut |f, item, _| write!(f, "{}", item.name),
          self.line(&span),
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
        self.write_statement_inline(body, indentation, false, f)?;
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
      if self.line(&prev.span) + 1 < self.line(&stmt.span) {
        writeln!(f)?;
      }

      self.fmt_statement(stmt, 0, true, f)?;
      prev = stmt;
    }

    Ok(())
  }
}

pub fn format(source: &str, ast: &[Statement]) -> String {
  Formatter::new(source, ast).to_string()
}
