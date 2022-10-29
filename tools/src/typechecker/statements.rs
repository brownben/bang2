use super::{
  builtins::ImportValue,
  types::{Truthiness, Type},
  Error, ErrorKind, HashMap, Typechecker,
};
use bang_syntax::ast::{
  expression::{Expression, LiteralType as Literal},
  statement::{DeclarationIdentifier, ImportItem, Statement},
  types::TypeExpression,
  Span,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReturnsLikelihood {
  Definite,
  Possible,
}
impl std::ops::BitAnd for ReturnsLikelihood {
  type Output = Self;

  fn bitand(self, rhs: Self) -> Self::Output {
    match (self, rhs) {
      (Self::Definite, Self::Definite) => Self::Definite,
      _ => Self::Possible,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatementType {
  Returns(Type, ReturnsLikelihood),
  NoReturn,
}

impl<'s> Typechecker<'s> {
  pub fn block_statement(&mut self, body: &[Statement<'s>]) -> Result<StatementType, Error> {
    self.scope.begin_scope();

    let mut return_type = Type::Union(Vec::new());
    let mut has_return = false;
    for statement in body {
      match self.synthesize_statement(statement)? {
        ty @ StatementType::Returns(_, ReturnsLikelihood::Definite) => {
          self.scope.end_scope();
          return Ok(ty);
        }
        StatementType::Returns(ty, ReturnsLikelihood::Possible) => {
          return_type = return_type.union(ty);
          has_return = true;
        }
        StatementType::NoReturn => {}
      };
    }

    self.scope.end_scope();

    if has_return {
      Ok(StatementType::Returns(
        return_type,
        ReturnsLikelihood::Possible,
      ))
    } else {
      Ok(StatementType::NoReturn)
    }
  }

  pub fn declaration_statement(
    &mut self,
    type_: &Option<TypeExpression<'s>>,
    expression: &Option<Expression<'s>>,
    identifier: &DeclarationIdentifier<'s>,
    span: Span,
  ) -> Result<StatementType, Error> {
    let annotation = if let Some(annotation) = type_ {
      self.type_from_annotation(annotation, &mut HashMap::new())?
    } else if let Some(expression) = expression {
      self.synthesize_expression(expression)?.uplevel_boolean()
    } else {
      Type::NULL
    };

    let ty = if let Some(expression) = expression {
      self.synthesize_expression(expression)?
    } else {
      Type::NULL
    };

    let ty = self.assert_type(ty, &annotation, span)?;

    match identifier {
      DeclarationIdentifier::Variable(identifier) => {
        self.scope.define(identifier, annotation.clone(), span)?;

        if annotation != ty {
          self.scope.update(identifier, ty);
        }
      }
      DeclarationIdentifier::List(identifiers) => {
        if annotation == Type::Literal(Literal::String) {
          for identifier in identifiers {
            self
              .scope
              .define(identifier, Type::Literal(Literal::String), span)?;
          }
        } else {
          let list_inner_type = self.context.new_existential();
          let list_type = Type::List(list_inner_type.clone().into());
          self.assert_type(annotation, &list_type, span)?;

          for identifier in identifiers {
            self
              .scope
              .define(identifier, list_inner_type.clone(), span)?;
          }
        }
      }
    };

    Ok(StatementType::NoReturn)
  }

  pub fn if_statement(
    &mut self,
    condition: &Expression<'s>,
    then: &Statement<'s>,
    otherwise: &Option<Box<Statement<'s>>>,
  ) -> Result<StatementType, Error> {
    use StatementType::{NoReturn, Returns};

    let condition_ty = self.synthesize_expression(condition)?;
    let restrictions = self.get_restrictions(condition)?;

    let then_ty = self.synthesize_statement_with_restrictions(then, restrictions)?;
    let else_ty = if let Some(otherwise) = otherwise {
      let restrictions = self.get_inverse_restrictions(condition)?;
      self.synthesize_statement_with_restrictions(otherwise, restrictions)?
    } else {
      StatementType::NoReturn
    };

    Ok(match condition_ty.truthiness() {
      Truthiness::True => then_ty,
      Truthiness::False => else_ty,
      Truthiness::Unknown => match (then_ty, else_ty) {
        (Returns(a, c), Returns(b, d)) => Returns(a.union(b), c & d),
        (Returns(a, _), NoReturn) | (NoReturn, Returns(a, _)) => {
          Returns(a, ReturnsLikelihood::Possible)
        }
        (NoReturn, NoReturn) => NoReturn,
      },
    })
  }

  pub fn import_statement(
    &mut self,
    items: &[ImportItem<'s>],
    module: &str,
  ) -> Result<StatementType, Error> {
    for item in items {
      match self.get_module_item(module, item.name) {
        ImportValue::Value(ty) => {
          self
            .scope
            .define(item.alias.unwrap_or(item.name), ty, item.span)?;
        }
        ImportValue::ModuleNotFound => {
          Err(Error::new(
            ErrorKind::ImportModuleNotFound(module.to_string()),
            item.span,
          ))?;
        }
        ImportValue::ItemNotFound => {
          Err(Error::new(
            ErrorKind::ImportItemNotFound(item.name.to_string()),
            item.span,
          ))?;
        }
      }
    }

    Ok(StatementType::NoReturn)
  }

  pub fn return_statement(
    &mut self,
    expression: &Option<Expression<'s>>,
  ) -> Result<StatementType, Error> {
    let ty = if let Some(expression) = expression {
      self.synthesize_expression(expression)?
    } else {
      Type::NULL
    };

    Ok(StatementType::Returns(ty, ReturnsLikelihood::Definite))
  }

  pub fn while_statement(
    &mut self,
    condition: &Expression<'s>,
    statement: &Statement<'s>,
  ) -> Result<StatementType, Error> {
    let condition_type = self.synthesize_expression(condition)?;
    let restrictions = self.get_restrictions(condition)?;

    if condition_type.truthiness() == Truthiness::False {
      Ok(StatementType::NoReturn)
    } else {
      let ty = self.synthesize_statement_with_restrictions(statement, restrictions)?;

      if ty == StatementType::NoReturn && condition_type.is_truthy() {
        Err(Error::new(ErrorKind::InfiniteLoop, condition.span))?;
      }

      Ok(match ty {
        StatementType::Returns(a, _) => StatementType::Returns(a, ReturnsLikelihood::Possible),
        StatementType::NoReturn => StatementType::NoReturn,
      })
    }
  }
}
