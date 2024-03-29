use super::{
  builtins::ImportValue,
  statements::StatementType,
  types::{Function, Literal, Type},
  Error, ErrorKind, HashMap, Typechecker,
};
use bang_syntax::ast::{
  expression::{operators, Expression, Parameter},
  statement::Statement,
  types::TypeExpression,
  Span,
};

impl<'s> Typechecker<'s> {
  pub fn assignment_expression(
    &mut self,
    expression: &Expression<'s>,
    identifier: &'s str,
    span: Span,
  ) -> Result<Type, Error> {
    let expression_ty = self.synthesize_expression(expression)?;
    let variable_ty = self
      .scope
      .lookup_initialization(identifier)
      .ok_or_else(|| {
        Error::new(ErrorKind::UndefinedVariable(identifier.to_string()), span).unwrap_err()
      })?;

    let expression_ty = self.assert_type(expression_ty, &variable_ty, span)?;
    self.scope.update(identifier, expression_ty.clone());
    Ok(expression_ty)
  }

  pub fn binary_expression(
    &mut self,
    operator: operators::Binary,
    left: &Expression<'s>,
    right: &Expression<'s>,
    span: Span,
  ) -> Result<Type, Error> {
    use operators::Binary as BinaryOperator;

    if operator == BinaryOperator::Pipeline {
      return self.synthesize_pipeline(left, right, span);
    }

    let l = self.synthesize_expression(left)?;
    let r = self.synthesize_expression(right)?;

    let ty = match operator {
      BinaryOperator::Plus => {
        self.assert_type(r, &l, span)?;
        self.assert_type(l, &Type::string_or_number(), span)?
      }
      BinaryOperator::Minus
      | BinaryOperator::Multiply
      | BinaryOperator::Divide
      | BinaryOperator::Remainder => {
        self.assert_type(l, &Type::Literal(Literal::Number), span)?;
        self.assert_type(r, &Type::Literal(Literal::Number), span)?
      }
      BinaryOperator::Equal | BinaryOperator::NotEqual => {
        self.assert_type(r, &l, span)?;
        Type::boolean()
      }
      BinaryOperator::Greater
      | BinaryOperator::Less
      | BinaryOperator::GreaterEqual
      | BinaryOperator::LessEqual => {
        self.assert_type(r, &l, span)?;
        self.assert_type(l, &Type::string_or_number(), span)?;
        Type::boolean()
      }
      BinaryOperator::And => match &l {
        type_ if type_.is_falsy() => l,
        type_ if type_.is_truthy() => r,
        _ => l.union(r),
      },
      BinaryOperator::Or => match &l {
        type_ if type_.is_falsy() => r,
        type_ if type_.is_truthy() => l,
        _ => l.union(r),
      },
      BinaryOperator::Nullish => {
        if self.subtype(&Type::NULL, &l) {
          l.narrow(&Type::NULL).union(r)
        } else {
          l
        }
      }
      BinaryOperator::Pipeline => unreachable!(),
    };

    Ok(ty)
  }

  pub fn function_expression(
    &mut self,
    name: Option<&'s str>,
    parameters: &[Parameter<'s>],
    body: &Statement<'s>,
    return_type: &Option<TypeExpression<'s>>,
    span: Span,
  ) -> Result<Type, Error> {
    let return_type = if let Some(return_type) = return_type {
      self.type_from_annotation(return_type, &mut HashMap::default())?
    } else {
      self.context.new_existential()
    };
    let arg_types = parameters
      .iter()
      .map(|param| {
        if let Some(ty) = &param.type_ {
          self.type_from_annotation(ty, &mut HashMap::default())
        } else {
          Ok(self.context.new_existential())
        }
      })
      .collect::<Result<Vec<_>, _>>()?;

    self.scope.begin_scope();

    for (type_, param) in arg_types.iter().cloned().zip(parameters.iter()) {
      self.scope.define(param.name, type_, span)?;
    }

    let function = Type::Function(Function {
      parameters: arg_types,
      return_type: return_type.clone().into(),
    });

    if let Some(name) = name && !self.scope.is_defined(name) {
      self.scope.define(name, function.clone(), span)?;
    };

    let ty = if let StatementType::Returns(ty, _) = self.synthesize_statement(body)? {
      ty
    } else {
      Type::NULL
    };
    self.assert_type(ty, &return_type, span)?;

    self.scope.end_scope();

    Ok(function)
  }

  pub fn list_expression(&mut self, items: &[Expression<'s>]) -> Result<Type, Error> {
    let inner_ty = if items.is_empty() {
      self.context.new_existential()
    } else {
      items
        .iter()
        .map(|item| self.synthesize_expression(item))
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter()
        .fold(Type::Never, Type::union)
    };

    Ok(Type::List(inner_ty.into()))
  }

  pub fn dictionary_expression(
    &mut self,
    items: &[(Expression<'s>, Expression<'s>)],
  ) -> Result<Type, Error> {
    let key_ty = if items.is_empty() {
      self.context.new_existential()
    } else {
      items
        .iter()
        .map(|item| self.synthesize_expression(&item.0))
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter()
        .fold(Type::Never, Type::union)
    };
    let value_ty = if items.is_empty() {
      self.context.new_existential()
    } else {
      items
        .iter()
        .map(|item| self.synthesize_expression(&item.1))
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter()
        .fold(Type::Never, Type::union)
    };

    Ok(Type::Dict(key_ty.into(), value_ty.into()))
  }

  pub fn index_expression(
    &mut self,
    index: &Expression<'s>,
    expression: &Expression<'s>,
    span: Span,
  ) -> Result<Type, Error> {
    let index_ty = self.synthesize_expression(index)?;
    let expression_ty = self.synthesize_expression(expression)?;

    let dict_type = Type::Dict(
      self.context.new_existential().into(),
      self.context.new_existential().into(),
    );

    if expression_ty == Type::Literal(Literal::String) {
      self.assert_type(index_ty, &Type::Literal(Literal::Number), span)?;
      Ok(Type::Literal(Literal::String))
    } else if !index_ty.is_subtype_of(&Type::Literal(Literal::Number))
      || expression_ty.is_subtype_of(&dict_type)
    {
      self.dict_index(index_ty, expression_ty, span)
    } else {
      self.list_index(index_ty, expression_ty, span)
    }
  }

  fn dict_index(&mut self, index_ty: Type, expression_ty: Type, span: Span) -> Result<Type, Error> {
    let keys = self.context.new_existential();
    let values = self.context.new_existential();
    let dict_ty = Type::Dict(Box::new(keys.clone()), Box::new(values.clone()));

    self.assert_type(index_ty, &keys, span)?;
    self.assert_type(expression_ty, &dict_ty, span)?;

    Ok(values.apply_context(&self.context))
  }

  fn list_index(&mut self, index_ty: Type, expression_ty: Type, span: Span) -> Result<Type, Error> {
    self.assert_type(index_ty, &Type::Literal(Literal::Number), span)?;

    let list_interior = self.context.new_existential();
    let list_ty = Type::List(Box::new(list_interior.clone()));
    self.assert_type(expression_ty, &list_ty, span)?;

    Ok(list_interior)
  }

  pub fn index_assgnment_expression(
    &mut self,
    expression: &Expression<'s>,
    index: &Expression<'s>,
    value: &Expression<'s>,
    assignment_operator: Option<operators::Assignment>,
    span: Span,
  ) -> Result<Type, Error> {
    let list_interior = self.context.new_existential();
    let list_ty = Type::List(Box::new(list_interior.clone()));

    let expression_ty = self.synthesize_expression(expression)?;
    let index_ty = self.synthesize_expression(index)?;
    let value_ty = self.synthesize_expression(value)?;

    self.assert_type(expression_ty, &list_ty, expression.span)?;
    self.assert_type(index_ty, &Type::Literal(Literal::Number), index.span)?;
    let value_ty = self.assert_type(value_ty, &list_interior, value.span)?;

    match assignment_operator {
      Some(operators::Assignment::Plus) => {
        self.assert_type(value_ty, &Type::string_or_number(), span)?;
        self.assert_type(list_interior.clone(), &Type::string_or_number(), span)?;
      }
      Some(_) => {
        self.assert_type(value_ty, &Type::Literal(Literal::Number), span)?;
        self.assert_type(list_interior.clone(), &Type::Literal(Literal::Number), span)?;
      }
      _ => (),
    };

    Ok(list_interior)
  }

  pub fn module_access(&mut self, module: &str, item: &str, span: Span) -> Result<Type, Error> {
    match self.get_module_item(module, item) {
      ImportValue::Value(ty) => Ok(ty),
      ImportValue::ModuleNotFound => {
        Error::new(ErrorKind::ImportModuleNotFound(module.to_string()), span)
      }
      ImportValue::ItemNotFound => {
        Error::new(ErrorKind::ImportItemNotFound(item.to_string()), span)
      }
    }
  }

  pub fn unary_expression(
    &mut self,
    expression: &Expression<'s>,
    operator: operators::Unary,
    span: Span,
  ) -> Result<Type, Error> {
    use operators::Unary;
    let ty = self.synthesize_expression(expression)?;

    match operator {
      Unary::Minus => self.assert_type(ty, &Type::Literal(Literal::Number), span),
      Unary::Not => Ok(match ty {
        ty if ty.is_truthy() => Type::Literal(Literal::False),
        ty if ty.is_falsy() => Type::Literal(Literal::True),
        _ => Type::boolean(),
      }),
    }
  }
}
