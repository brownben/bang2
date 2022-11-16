use super::{
  statements::StatementType,
  types::{Literal, Type},
  Error, HashMap, Typechecker,
};
use bang_syntax::ast::{
  expression::{operators, Expr, Expression},
  statement::Statement,
};
use std::collections::hash_map::Entry as HashMapEntry;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Restriction<'s>(&'s str, Type);

impl<'s> Typechecker<'s> {
  pub fn synthesize_statement_with_restrictions(
    &mut self,
    statement: &Statement<'s>,
    restrictions: Vec<Restriction<'s>>,
  ) -> Result<StatementType, Error> {
    self.context.begin_scope();
    self.scope.begin_scope();

    restrictions
      .into_iter()
      .for_each(|Restriction(name, ty)| self.scope.update(name, ty));

    let ty = self.synthesize_statement(statement);

    self.scope.end_scope();
    self.context.end_scope();

    ty
  }

  pub fn get_restrictions(
    &mut self,
    expression: &Expression<'s>,
  ) -> Result<Vec<Restriction<'s>>, Error> {
    match &expression.expr {
      Expr::Group { expression } | Expr::Comment { expression, .. } => {
        return self.get_restrictions(expression);
      }
      Expr::Binary {
        operator: operators::Binary::Equal,
        left,
        right,
      } => {
        let r = self.synthesize_expression(right)?;

        if let Expr::Variable { name } = &left.expr {
          return Ok(vec![Restriction(name, r)]);
        }
      }
      Expr::Binary {
        operator: operators::Binary::NotEqual,
        left,
        right,
      } => {
        let l = self.synthesize_expression(left)?;
        let r = self.synthesize_expression(right)?;

        if let Expr::Variable { name } = &left.expr
          && let Type::Literal(Literal::Null | Literal::True | Literal::False) = &r
        {
          return Ok(vec![Restriction(name, l.narrow(&r))]);
        }
      }
      Expr::Binary {
        operator: operators::Binary::And,
        left,
        right,
      } => {
        let l = self.get_restrictions(left)?;
        let r = self.get_restrictions(right)?;

        let mut restrictions = l
          .into_iter()
          .map(|Restriction(name, type_)| (name, type_))
          .collect::<HashMap<_, _>>();

        for Restriction(name, restriction) in r {
          if let HashMapEntry::Occupied(mut entry) = restrictions.entry(name) {
            entry.insert(restriction.narrow(entry.get()));
          } else {
            restrictions.insert(name, restriction);
          }
        }

        return Ok(
          restrictions
            .into_iter()
            .map(|(name, restriction)| Restriction(name, restriction))
            .collect::<Vec<_>>(),
        );
      }
      Expr::Binary {
        operator: operators::Binary::Or,
        left,
        right,
      } => {
        let mut restrictions = self.get_restrictions(left)?;
        restrictions.append(&mut self.get_restrictions(right)?);

        // If different variables are on each side of the OR we can't make a decision
        let first_name = restrictions[0].0;
        let all_same_name = restrictions
          .iter()
          .all(|Restriction(name, _)| *name == first_name);

        if all_same_name {
          return Ok(vec![Restriction(
            first_name,
            restrictions
              .into_iter()
              .map(|Restriction(_, t)| t)
              .fold(Type::Never, Type::union),
          )]);
        }
      }
      _ => {}
    }

    Ok(Vec::new())
  }

  pub fn get_inverse_restrictions(
    &mut self,
    expression: &Expression<'s>,
  ) -> Result<Vec<Restriction<'s>>, Error> {
    let restrictions = self.get_restrictions(expression)?;

    Ok(
      restrictions
        .into_iter()
        .map(|Restriction(name, not_type)| {
          let current_type = self
            .scope
            .lookup(name)
            .expect("variable to exist as previously found");

          Restriction(name, current_type.narrow(&not_type))
        })
        .collect(),
    )
  }
}
