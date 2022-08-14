use super::{Diagnostic, LintRule};
use bang_syntax::ast::{
  expression::{Expr, Expression},
  statement::{DeclarationIdentifier, Statement, Stmt},
  Span, Visitor,
};

struct Variable {
  name: String,
  span: Span,
  depth: u8,
  used: bool,
}

struct Scope {
  depth: u8,
  variables: Vec<Variable>,
}
impl Scope {
  pub fn begin(&mut self) {
    self.depth += 1;
  }

  pub fn end(&mut self) -> Vec<Span> {
    let mut unused = Vec::new();

    while let Some(last) = self.variables.last() && last.depth == self.depth {
      let variable = self.variables.pop();
      if let Some(Variable { used: false, span, name, .. }) = variable {
        if !name.starts_with('_') {
          unused.push(span);
        }
      }
    }

    self.depth -= 1;
    unused
  }

  pub fn define(&mut self, name: &str, span: Span) {
    self.variables.push(Variable {
      name: name.to_string(),
      span,
      depth: self.depth,
      used: false,
    });
  }

  pub fn used(&mut self, name: &str) {
    if let Some(index) = self.variables.iter().rposition(|local| local.name == name) {
      self.variables[index].used = true;
    }
  }
}
impl Default for Scope {
  fn default() -> Self {
    Self {
      depth: 1,
      variables: Vec::new(),
    }
  }
}

lint_rule! {
  name: NoUnusedVariables;
  title: "No Unused Variables";
  message: "Variables have been defined but are never accessed";
  data: Scope;
  visitor: {
    fn enter_statement(&mut self, statement: &Statement) {
      if let Stmt::Block { .. } = statement.stmt {
        self.data.begin();
      }
    }

    fn exit_statement(&mut self, statement: &Statement) {
      if let Stmt::Block { .. } = statement.stmt {
        self.issues.extend(self.data.end().iter());
      }

      if let Stmt::Declaration { identifier, .. } = &statement.stmt {
        match identifier {
          DeclarationIdentifier::Variable(name) => self.data.define(name, statement.span),
          DeclarationIdentifier::List(names) => names
            .iter()
            .for_each(|name| self.data.define(name, statement.span)),
        }
      }
    }

    fn exit_expression(&mut self, expression: &Expression) {
      if let Expr::Variable { name } = expression.expr {
        self.data.used(name);
      }

      if let Expr::Function { parameters, .. } = &expression.expr {
        parameters.iter().for_each(|param| self.data.define(param.name, param.span));
      }
    }

    fn exit_ast(&mut self) {
      self.issues.extend(self.data.end().iter());
    }
  }
}
