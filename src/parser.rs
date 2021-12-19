use crate::{
  ast::{BinaryOperator, Expression, LiteralValue, Parameter, Statement, UnaryOperator},
  error::{CompileError, Error},
  scanner::Scanner,
  token::{Token, TokenType},
};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::rc::Rc;

#[derive(Debug, FromPrimitive, PartialOrd, PartialEq, Clone, Copy)]
enum Precedence {
  None = 1,
  Assignment, // =
  Or,         // or
  And,        // and
  Nullish,    // ??
  Equality,   // == !=
  Comparison, // < > <= >=
  Term,       // + -
  Factor,     // * /
  Unary,      // ! -
  Call,       // . ()
  Primary,
}

fn get_precedence(number: u8) -> Precedence {
  match FromPrimitive::from_u8(number) {
    Some(precedence) => precedence,
    _ => Precedence::None,
  }
}

type ExpressionResult = Result<Expression, CompileError>;
type StatementResult = Result<Statement, CompileError>;

type ParsePrefixFn = fn(parser: &mut Parser, can_assign: bool) -> ExpressionResult;
type ParseInfixFn =
  fn(parser: &mut Parser, previous: Expression, can_assign: bool) -> ExpressionResult;

struct ParseRule {
  pub prefix: Option<ParsePrefixFn>,
  pub infix: Option<ParseInfixFn>,
  pub precedence: Precedence,
}

fn get_rule(token_type: TokenType) -> ParseRule {
  match token_type {
    TokenType::LeftParen => ParseRule {
      prefix: Some(grouping),
      infix: Some(call),
      precedence: Precedence::Call,
    },

    TokenType::Plus => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Term,
    },
    TokenType::Minus => ParseRule {
      prefix: Some(unary),
      infix: Some(binary),
      precedence: Precedence::Term,
    },
    TokenType::Star | TokenType::Slash => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Factor,
    },

    TokenType::Bang => ParseRule {
      prefix: Some(unary),
      infix: None,
      precedence: Precedence::None,
    },
    TokenType::BangEqual | TokenType::EqualEqual => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Equality,
    },
    TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual => {
      ParseRule {
        prefix: None,
        infix: Some(binary),
        precedence: Precedence::Comparison,
      }
    }

    TokenType::Identifier => ParseRule {
      prefix: Some(variable),
      infix: None,
      precedence: Precedence::None,
    },
    TokenType::String => ParseRule {
      prefix: Some(string),
      infix: None,
      precedence: Precedence::None,
    },
    TokenType::Number => ParseRule {
      prefix: Some(number),
      infix: None,
      precedence: Precedence::None,
    },
    TokenType::True | TokenType::False | TokenType::Null => ParseRule {
      prefix: Some(literal),
      infix: None,
      precedence: Precedence::None,
    },

    TokenType::And => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::And,
    },
    TokenType::Or => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Or,
    },
    TokenType::QuestionQuestion => ParseRule {
      prefix: None,
      infix: Some(binary),
      precedence: Precedence::Nullish,
    },

    _ => ParseRule {
      prefix: None,
      infix: None,
      precedence: Precedence::None,
    },
  }
}

struct Parser {
  scanner: Scanner,
  backlog: Vec<Token>,

  current: Option<Token>,
  previous: Option<Token>,
}

impl Parser {
  fn new(source: &str) -> Self {
    Self {
      scanner: Scanner::new(source),
      backlog: Vec::new(),

      current: None,
      previous: None,
    }
  }

  fn error(&mut self, token: Token, error: Error) -> CompileError {
    CompileError { error, token }
  }

  fn advance(&mut self) -> Result<(), CompileError> {
    self.previous = self.current.take();
    let token = if self.backlog.is_empty() {
      self.scanner.get_token()
    } else {
      self.backlog.remove(0)
    };

    if token.token_type == TokenType::Error {
      Err(self.error(token, token.error_value.unwrap()))
    } else {
      self.current = Some(token);
      Ok(())
    }
  }

  fn matches(&mut self, token_type: TokenType) -> bool {
    if self.current.is_some() && self.current.unwrap().token_type == token_type {
      let result = self.advance();
      matches!(result, Ok(()))
    } else {
      false
    }
  }

  fn get_token_or_backlog(&mut self, initial: usize, count: usize) -> Token {
    if count < initial {
      self.backlog[count]
    } else {
      let token = self.scanner.get_token();
      self.backlog.push(token);
      token
    }
  }

  fn is_function_bracket(&mut self) -> bool {
    let initial_depth = self.backlog.len();

    let mut token = self.current.unwrap();
    let mut depth = 0;
    let mut count = 0;

    loop {
      if depth == 0 && token.token_type == TokenType::RightParen {
        break;
      } else if token.token_type == TokenType::EndOfFile {
        return false;
      } else if token.token_type == TokenType::RightParen {
        depth += 1;
      } else if token.token_type == TokenType::LeftParen {
        depth -= 1;
      }

      token = self.get_token_or_backlog(initial_depth, count);
      count += 1;
    }

    token = self.get_token_or_backlog(initial_depth, count);
    token.token_type == TokenType::FatRightArrow || token.token_type == TokenType::RightArrow
  }

  fn consume(&mut self, token_type: TokenType, message: Error) -> Result<(), CompileError> {
    let current = self.current.unwrap();
    if current.token_type == token_type {
      self.advance()
    } else {
      Err(self.error(current, message))
    }
  }

  fn parse(&mut self, precedence: Precedence) -> ExpressionResult {
    self.advance()?;

    let token = self.previous.unwrap();
    let prefix_rule = get_rule(token.token_type).prefix;
    if prefix_rule.is_none() {
      return Err(self.error(token, Error::ExpectedExpression));
    }

    let can_assign = precedence <= Precedence::Assignment;
    let mut previous = prefix_rule.unwrap()(self, can_assign)?;

    while precedence <= get_rule(self.current.unwrap().token_type).precedence {
      self.advance()?;

      if let Some(infix_rule) = get_rule(self.previous.unwrap().token_type).infix {
        previous = infix_rule(self, previous, can_assign)?;
      }
    }

    if can_assign && self.matches(TokenType::Equal) {
      Err(self.error(token, Error::InvalidAssignmentTarget))
    } else {
      Ok(previous)
    }
  }
}

pub fn parse(source: &str) -> Result<Vec<Statement>, CompileError> {
  let mut parser = Parser::new(source);

  parser.advance()?;

  let mut statements = Vec::new();
  while !parser.matches(TokenType::EndOfFile) {
    statements.push(statement(&mut parser)?);
  }

  parser.consume(TokenType::EndOfFile, Error::MissingEndOfFile)?;

  Ok(statements)
}

fn statement(parser: &mut Parser) -> StatementResult {
  if parser.matches(TokenType::Let) {
    var_declaration(parser)
  } else if parser.matches(TokenType::Return) {
    return_statement(parser)
  } else if parser.matches(TokenType::BlockStart) {
    block(parser)
  } else if parser.matches(TokenType::While) {
    while_statement(parser)
  } else if parser.matches(TokenType::If) {
    if_statement(parser)
  } else {
    expression_statement(parser)
  }
}

fn var_declaration(parser: &mut Parser) -> StatementResult {
  let token = parser.current.unwrap();
  parser.consume(TokenType::Identifier, Error::MissingVariableName)?;
  let identifier = parser.previous.unwrap();
  let variable_name = identifier.get_value(&parser.scanner.chars);

  let expression = if parser.matches(TokenType::Equal) {
    Some(expression(parser)?)
  } else {
    None
  };
  parser.consume(TokenType::EndOfLine, Error::ExpectedNewLine)?;

  Ok(Statement::Declaration {
    token,
    identifier,
    variable_name,
    expression,
  })
}

fn block(parser: &mut Parser) -> StatementResult {
  let mut statements = Vec::new();
  while parser.current.unwrap().token_type != TokenType::BlockEnd {
    statements.push(statement(parser)?);
  }
  parser.consume(TokenType::BlockEnd, Error::ExpectedEndOfBlock)?;

  Ok(Statement::Block { body: statements })
}

fn return_statement(parser: &mut Parser) -> StatementResult {
  let token = parser.current.unwrap();

  let expression = if parser.matches(TokenType::EndOfLine) {
    None
  } else {
    let exp = Some(expression(parser)?);
    parser.consume(TokenType::EndOfLine, Error::ExpectedNewLine)?;
    exp
  };

  Ok(Statement::Return { token, expression })
}

fn if_statement(parser: &mut Parser) -> StatementResult {
  let if_token = parser.current.unwrap();
  parser.consume(TokenType::LeftParen, Error::MissingBracketBeforeCondition)?;
  let condition = expression(parser)?;
  parser.consume(TokenType::RightParen, Error::MissingBracketAfterCondition)?;

  parser.matches(TokenType::EndOfLine);
  let body = statement(parser)?;

  let mut else_token = None;
  let otherwise = if parser.matches(TokenType::Else) {
    else_token = parser.previous;
    Some(Box::new(statement(parser)?))
  } else {
    None
  };

  Ok(Statement::If {
    if_token,
    else_token,
    condition,
    then: Box::new(body),
    otherwise,
  })
}

fn while_statement(parser: &mut Parser) -> StatementResult {
  let token = parser.current.unwrap();

  parser.consume(TokenType::LeftParen, Error::MissingBracketBeforeCondition)?;
  let condition = expression(parser)?;
  parser.consume(TokenType::RightParen, Error::MissingBracketAfterCondition)?;

  parser.matches(TokenType::EndOfLine);
  let body = Box::new(statement(parser)?);

  Ok(Statement::While {
    token,
    condition,
    body,
  })
}

fn expression_statement(parser: &mut Parser) -> StatementResult {
  let expression = expression(parser)?;
  parser.consume(TokenType::EndOfLine, Error::ExpectedNewLine)?;

  Ok(Statement::Expression { expression })
}

fn expression(parser: &mut Parser) -> ExpressionResult {
  parser.parse(Precedence::Assignment)
}

fn variable(parser: &mut Parser, can_assign: bool) -> ExpressionResult {
  let identifier = parser.previous.unwrap();
  let name = identifier.get_value(&parser.scanner.chars);

  let additional_operator = match parser.current.unwrap().token_type {
    TokenType::PlusEqual | TokenType::MinusEqual | TokenType::StarEqual | TokenType::SlashEqual => {
      parser.current
    }
    _ => None,
  };

  if let (true, Some(token)) = (can_assign, additional_operator) {
    parser.advance()?;
    let expression = expression(parser)?;
    let operator = match token.token_type {
      TokenType::PlusEqual => BinaryOperator::Plus,
      TokenType::MinusEqual => BinaryOperator::Minus,
      TokenType::StarEqual => BinaryOperator::Star,
      TokenType::SlashEqual => BinaryOperator::Slash,
      _ => unreachable!(),
    };

    Ok(Expression::Assignment {
      identifier,
      variable_name: name.clone(),
      expression: Box::new(Expression::Binary {
        token,
        left: Box::new(Expression::Variable {
          identifier,
          variable_name: name,
        }),
        operator,
        right: Box::new(expression),
      }),
    })
  } else if parser.matches(TokenType::Equal) && can_assign {
    let expression = expression(parser)?;
    Ok(Expression::Assignment {
      identifier,
      variable_name: name,
      expression: Box::new(expression),
    })
  } else {
    Ok(Expression::Variable {
      identifier,
      variable_name: name,
    })
  }
}

fn grouping(parser: &mut Parser, _can_assign: bool) -> ExpressionResult {
  if parser.is_function_bracket() {
    return function(parser);
  }

  let expression = expression(parser)?;
  parser.consume(TokenType::RightParen, Error::ExpectedBracket)?;

  Ok(Expression::Group {
    expression: Box::new(expression),
  })
}

fn function(parser: &mut Parser) -> ExpressionResult {
  let token = parser.current.unwrap();

  let mut parameters = Vec::new();
  if token.token_type != TokenType::RightParen {
    loop {
      if parser.current.unwrap().token_type == TokenType::RightParen {
        break;
      }

      parser.consume(TokenType::Identifier, Error::MissingVariableName)?;
      let token = parser.previous.unwrap();
      parser.consume(TokenType::Colon, Error::MissingColonBeforeType)?;
      parser.consume(TokenType::Identifier, Error::MissingTypeName)?;
      let type_token = parser.previous.unwrap();

      parameters.push(Parameter {
        identifier: token,
        value: token.get_value(&parser.scanner.chars),
        type_: type_token.get_value(&parser.scanner.chars),
      });

      if !parser.matches(TokenType::Comma) {
        break;
      }
    }
  }
  parser.consume(TokenType::RightParen, Error::ExpectedBracket)?;

  let (return_type, body) = if parser.matches(TokenType::RightArrow) {
    parser.consume(TokenType::Identifier, Error::MissingTypeName)?;
    let return_type = parser.previous.unwrap().get_value(&parser.scanner.chars);
    parser.matches(TokenType::EndOfLine);

    parser.consume(TokenType::BlockStart, Error::ExpectedStartOfBlock)?;

    let mut statements = Vec::new();
    while parser.current.unwrap().token_type != TokenType::BlockEnd {
      statements.push(statement(parser)?);
    }

    // Make sure that the expression always ends with a end of line
    if let Some(mut current) = parser.current {
      current.token_type = TokenType::EndOfLine;
      parser.current = Some(current);
    }

    (Some(return_type), Statement::Block { body: statements })
  } else {
    parser.consume(TokenType::FatRightArrow, Error::MissingFunctionArrow)?;
    (
      None,
      Statement::Return {
        token: parser.current.unwrap(),
        expression: Some(expression(parser)?),
      },
    )
  };

  Ok(Expression::Function {
    token,
    body: Box::new(body),
    parameters,
    return_type,
  })
}

fn unary(parser: &mut Parser, _can_assign: bool) -> ExpressionResult {
  let token = parser.previous.unwrap();
  let operator = match token.token_type {
    TokenType::Minus => UnaryOperator::Minus,
    TokenType::Bang => UnaryOperator::Bang,
    _ => unreachable!(),
  };
  let expression = parser.parse(Precedence::Unary)?;

  Ok(Expression::Unary {
    token,
    operator,
    expression: Box::new(expression),
  })
}

fn call(parser: &mut Parser, previous: Expression, _can_assign: bool) -> ExpressionResult {
  let token = parser.previous.unwrap();
  let mut arguments = Vec::new();

  if parser.current.unwrap().token_type != TokenType::RightParen {
    loop {
      if parser.current.unwrap().token_type == TokenType::RightParen {
        break;
      }

      arguments.push(expression(parser)?);

      if !parser.matches(TokenType::Comma) {
        break;
      }
    }
  }

  parser.consume(TokenType::RightParen, Error::ExpectedBracket)?;

  Ok(Expression::Call {
    expression: Box::new(previous),
    token,
    arguments,
  })
}

fn binary(parser: &mut Parser, previous: Expression, _can_assign: bool) -> ExpressionResult {
  let token = parser.previous.unwrap();
  let operator = match token.token_type {
    TokenType::Plus => BinaryOperator::Plus,
    TokenType::Minus => BinaryOperator::Minus,
    TokenType::Star => BinaryOperator::Star,
    TokenType::Slash => BinaryOperator::Slash,
    TokenType::BangEqual => BinaryOperator::BangEqual,
    TokenType::EqualEqual => BinaryOperator::EqualEqual,
    TokenType::Greater => BinaryOperator::Greater,
    TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
    TokenType::Less => BinaryOperator::Less,
    TokenType::LessEqual => BinaryOperator::LessEqual,
    TokenType::And => BinaryOperator::And,
    TokenType::Or => BinaryOperator::Or,
    TokenType::QuestionQuestion => BinaryOperator::QuestionQuestion,
    _ => unreachable!(),
  };
  let rule = get_rule(token.token_type);
  let right = parser.parse(get_precedence((rule.precedence as u8) + 1))?;

  Ok(Expression::Binary {
    token,
    left: Box::new(previous),
    operator,
    right: Box::new(right),
  })
}

fn string(parser: &mut Parser, _can_assign: bool) -> ExpressionResult {
  let token = parser.previous.unwrap();
  let token_value = token.get_value(&parser.scanner.chars);
  let value = token_value[1..token_value.len() - 1].to_string();

  Ok(Expression::Literal {
    token,
    value: LiteralValue::String(Rc::from(value)),
  })
}

fn number(parser: &mut Parser, _can_assign: bool) -> ExpressionResult {
  let token = parser.previous.unwrap();
  let value: f64 = token
    .get_value(&parser.scanner.chars)
    .replace('_', "")
    .parse()
    .unwrap();

  Ok(Expression::Literal {
    token,
    value: LiteralValue::Number(value),
  })
}

fn literal(parser: &mut Parser, _can_assign: bool) -> ExpressionResult {
  let token = parser.previous.unwrap();

  let value = match token.token_type {
    TokenType::True => LiteralValue::True,
    TokenType::False => LiteralValue::False,
    _ => LiteralValue::Null,
  };

  Ok(Expression::Literal { value, token })
}
