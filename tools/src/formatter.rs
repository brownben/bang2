use bang_syntax::{
  ast::{
    expression::{AssignmentOperator, Expr, Expression, LiteralType},
    statement::{DeclarationIdentifier, Statement, Stmt},
    types::{Type, TypeExpression},
    Span,
  },
  parse_number, LineNumber,
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

  fn line(&self, span: Span) -> LineNumber {
    span.get_line_number(self.source)
  }

  fn line_end(&self, span: Span) -> LineNumber {
    span.get_line_number_end(self.source)
  }

  fn write_group(
    &self,
    expression: &Expression,
    indentation: usize,
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    write!(f, "(")?;
    if self.line(expression.span) == self.line_end(expression.span) {
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
    f: &mut std::fmt::Formatter,
  ) -> std::fmt::Result {
    if let Stmt::Block { body, .. } = &statement.stmt {
      if body.len() > 1 {
        writeln!(f)?;
        self.fmt_statement(statement, indentation, true, f)?;
      } else {
        write!(f, " ")?;
        self.fmt_statement(&body[0], indentation, false, f)?;
      }
    } else {
      write!(f, " ")?;
      self.fmt_statement(statement, indentation, false, f)?;
    }

    Ok(())
  }

  fn write_list<Item>(
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

  fn fmt_type(&self, t: &TypeExpression, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match &t.type_ {
      Type::Named(name) => write!(f, "{name}")?,
      Type::Union(a, b) => {
        self.fmt_type(a, f)?;
        write!(f, " | ")?;
        self.fmt_type(b, f)?;
      }
      Type::Function(return_type, parameters) => {
        write!(f, "(")?;
        for (i, param) in parameters.iter().enumerate() {
          self.fmt_type(param, f)?;
          if i < parameters.len() - 1 {
            write!(f, ", ")?;
          }
        }
        write!(f, ") -> ")?;
        self.fmt_type(return_type, f)?;
      }
      Type::Group(type_) => {
        write!(f, "(")?;
        self.fmt_type(type_, f)?;
        write!(f, ")")?;
      }
      Type::Optional(type_) => {
        self.fmt_type(type_, f)?;
        write!(f, "?")?;
      }
      Type::List(type_) => {
        self.fmt_type(type_, f)?;
        write!(f, "[]")?;
      }
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
        Self::write_list(
          arguments,
          |arg| self.line(arg.span),
          &mut |f, arg, i| self.fmt_expression(arg, i, f),
          self.line(expression.span),
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
      Expr::FormatString { expressions, strings } => {
        write!(f, "'{}", strings[0])?;
        for (index, expression) in expressions.iter().enumerate() {
          write!(f, "${{")?;
          self.fmt_expression(expression, indentation, f)?;
          write!(f, "}}{}", strings[index + 1])?;
        }
        write!(f, "'")?;
      },
      Expr::Function {
        parameters,
        body,
        return_type,
        ..
      } => {
        write!(f, "(")?;
        Self::write_list(
          parameters,
          |param| self.line(param.span),
          &mut |f, parameter, _| {
            write!(f, "{}", parameter.name)?;

            if let Some(type_) = &parameter.type_ {
              write!(f, ": ")?;
              self.fmt_type(type_, f)?;
            }
            Ok(())
          },
          self.line(span),
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
          write!(f, ") ->")?;
          if let Some(return_type) = return_type {
            write!(f, " ")?;
            self.fmt_type(return_type, f)?;
          }
          writeln!(f)?;
          self.fmt_statement(body, indentation, false, f)?;
        }
      }
      Expr::Group { expression, .. } => {
        self.write_group( expression, indentation, f)?;
      }
      Expr::Index { expression, index } => {
        self.fmt_expression(expression, indentation, f)?;
        write!(f, "[")?;
        self.fmt_expression(index, indentation, f)?;
        write!(f, "]")?;
      }
      Expr::IndexAssignment { expression, index, value, assignment_operator } => {
        self.fmt_expression(expression, indentation, f)?;
        write!(f, "[")?;
        self.fmt_expression(index, indentation, f)?;
        write!(f, "] {} ", assignment_operator.map(|operator| operator.to_string()).unwrap_or_else(|| "=".to_string()))?;
        self.fmt_expression(value, indentation, f)?;
      }
      Expr::List {
        items
      } => {
        write!(f, "[")?;
        Self::write_list(
          items,
          |item| self.line(item.span),
          &mut |f, item, i| self.fmt_expression(item, i, f),
          self.line(expression.span),
          indentation,
          false,
          f,
        )?;
        write!(f, "]")?;
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
          if !body.is_empty() {
            let mut prev = &body[0];
            for stmt in body {
              if self.line_end(prev.span) + 1 < self.line(stmt.span) {
                writeln!(f)?;
              }

              write!(f, "{}", INDENTATION.repeat(indentation + 1))?;
              self.fmt_statement(stmt, indentation + 1, true, f)?;
              prev = stmt;
            }

            if self.line_end(prev.span) + 1 < self.line(last.span) {
              writeln!(f)?;
            }
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
        type_,
      } => {
        match identifier {
          DeclarationIdentifier::Variable(identifier) => write!(f, "let {identifier}")?,
          DeclarationIdentifier::List(list) => {
            write!(f, "let [")?;
            Self::write_list(
              list,
              |_| self.line(span),
              &mut |f, item, _| write!(f, "{item}"),
              self.line(span),
              indentation,
              false,
              f,
            )?;
            write!(f, "]")?;
          }
        };

        if let Some(type_) = type_ {
          write!(f, ": ")?;
          self.fmt_type(type_, f)?;
        }
        if let Some(expression) = expression {
          write!(f, " = ")?;
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
        self.write_statement_inline(then, indentation, f)?;

        if let Some(otherwise) = otherwise {
          write!(f, "\n{}else", INDENTATION.repeat(indentation))?;
          self.write_statement_inline(otherwise, indentation, f)?;
        }
      }
      Stmt::Import { module, items, .. } => {
        write!(f, "from {module} import {{")?;
        Self::write_list(
          items,
          |item| self.line(item.span),
          &mut |f, item, _| {
            write!(f, "{}", item.name)?;
            if let Some(alias) = &item.alias {
              write!(f, " as {alias}")?;
            }
            Ok(())
          },
          self.line(span),
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
        self.write_statement_inline(body, indentation, f)?;
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
      if self.line_end(prev.span) < self.line(stmt.span) {
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
