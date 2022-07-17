use crate::{
  ast::{
    expression::{
      expression, AssignmentOperator, BinaryOperator, Expr, Expression, LiteralType, Parameter,
      UnaryOperator,
    },
    statement::{statement, DeclarationIdentifier, ImportItem, Statement, Stmt},
    types::{types, Type, TypeExpression},
    Span,
  },
  tokens::{tokenize, Token, TokenType},
  LineNumber,
};
use std::{error, fmt};

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq)]
enum Precedence {
  None = 1,
  Assignment, // =
  Pipeline,   // >>
  Or,         // or
  And,        // and
  Nullish,    // ??
  Equality,   // == !=
  Comparison, // < > <= >=
  Term,       // + -
  Factor,     // * /
  Unary,      // ! -
  Call,       // () []
  Primary,
  Comment,
}

impl Precedence {
  fn next(self) -> Self {
    match self {
      Self::None => Self::Assignment,
      Self::Assignment => Self::Pipeline,
      Self::Pipeline => Self::Or,
      Self::Or => Self::And,
      Self::And => Self::Nullish,
      Self::Nullish => Self::Equality,
      Self::Equality => Self::Comparison,
      Self::Comparison => Self::Term,
      Self::Term => Self::Factor,
      Self::Factor => Self::Unary,
      Self::Unary => Self::Call,
      Self::Call | Self::Primary | Self::Comment => Self::Primary,
    }
  }

  fn from(token_type: TokenType) -> Self {
    match token_type {
      TokenType::And => Self::And,
      TokenType::Or => Self::Or,
      TokenType::QuestionQuestion => Self::Nullish,
      TokenType::LeftParen | TokenType::LeftSquare => Self::Call,
      TokenType::Plus | TokenType::Minus => Self::Term,
      TokenType::Star | TokenType::Slash => Self::Factor,
      TokenType::BangEqual | TokenType::EqualEqual => Self::Equality,
      TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual => {
        Self::Comparison
      }
      TokenType::Comment => Self::Comment,
      TokenType::RightRight => Self::Pipeline,
      _ => Self::None,
    }
  }
}

enum Error {
  ExpectedOpeningBracket,
  ExpectedClosingBracket,
  ExpectedOpeningBrace,
  ExpectedClosingBrace,
  ExpectedClosingSquare,
  ExpectedExpression,
  ExpectedFunctionArrow,
  ExpectedNewLine,
  ExpectedIdentifier,
  InvalidAssignmentTarget,
  UnexpectedCharacter,
  UnterminatedString,
  EmptyStatement,
  ExpectedImportKeyword,
  ExpectedType,
  ExpectedCatchAllLast,
}
impl Error {
  fn get_title(&self) -> &'static str {
    match self {
      Self::ExpectedOpeningBracket => "Expected '('",
      Self::ExpectedClosingBracket => "Expected ')'",
      Self::ExpectedOpeningBrace => "Expected '{'",
      Self::ExpectedClosingBrace => "Expected '}'",
      Self::ExpectedClosingSquare => "Expected ']'",
      Self::ExpectedExpression => "Expected Expression",
      Self::ExpectedFunctionArrow => "Expected Function Arrow (-> / =>)",
      Self::ExpectedNewLine => "Expected New Line",
      Self::ExpectedIdentifier => "Expected Identifier",
      Self::InvalidAssignmentTarget => "Invalid Assignment Target",
      Self::UnexpectedCharacter => "Unexpected Character",
      Self::UnterminatedString => "Unterminated String",
      Self::ExpectedImportKeyword => "Expected 'import' keyword",
      Self::ExpectedType => "Expected Type",
      Self::ExpectedCatchAllLast => "Expected Catch All Parameter To Be The Last",
      Self::EmptyStatement => unreachable!("EmptyStatement caught to return nothing"),
    }
  }

  fn get_message(&self, source: &[u8], token: &Token) -> String {
    match self {
      Self::ExpectedOpeningBracket
      | Self::ExpectedClosingBracket
      | Self::ExpectedOpeningBrace
      | Self::ExpectedClosingBrace
      | Self::ExpectedClosingSquare
      | Self::ExpectedExpression
      | Self::ExpectedFunctionArrow
      | Self::ExpectedNewLine
      | Self::ExpectedIdentifier
      | Self::ExpectedImportKeyword
      | Self::ExpectedType => format!("but recieved '{}'", token.get_value(source)),
      Self::UnexpectedCharacter => format!("Unknown character '{}'", token.get_value(source)),
      Self::UnterminatedString => {
        format!("Missing closing quote {}", &token.get_value(source)[0..1])
      }
      Self::InvalidAssignmentTarget => "Can't assign to an expression, only a variable".to_string(),
      Self::ExpectedCatchAllLast => "No parameters can follow a catch all parameter".to_string(),
      Self::EmptyStatement => unreachable!("EmptyStatement caught to return nothing"),
    }
  }

  fn get_diagnostic(&self, source: &str, token: &Token) -> Diagnostic {
    let span: Span = token.into();

    Diagnostic {
      title: self.get_title().to_string(),
      message: self.get_message(source.as_bytes(), token),
      line: span.get_line_number(source),
      span,
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Diagnostic {
  pub title: String,
  pub message: String,
  pub span: Span,
  pub line: LineNumber,
}
impl fmt::Display for Diagnostic {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Error: {}\n\t{}\nat line {}",
      self.title, self.message, self.line
    )
  }
}
impl error::Error for Diagnostic {}

type ExpressionResult<'source> = Result<Expression<'source>, Error>;
type StatementResult<'source> = Result<Statement<'source>, Error>;
type TypeResult<'source> = Result<TypeExpression<'source>, Error>;

struct Parser<'source, 'tokens> {
  source: &'source [u8],
  tokens: &'tokens [Token],
  position: usize,
}

impl<'source, 'tokens> Parser<'source, 'tokens> {
  fn new(source: &'source str, tokens: &'tokens [Token]) -> Self {
    Self {
      source: source.as_bytes(),
      tokens,
      position: 0,
    }
  }

  fn at_end(&self) -> bool {
    self.position >= self.tokens.len()
      || self.current().ttype == TokenType::EndOfFile
      || self.peek() == TokenType::EndOfFile
  }

  fn next(&mut self) -> &'tokens Token {
    self.position += 1;
    let token = self.current();

    if token.ttype == TokenType::Whitespace {
      self.next()
    } else {
      token
    }
  }

  fn peek(&self) -> TokenType {
    let mut position = self.position + 1;
    if position >= self.tokens.len() {
      position = self.tokens.len() - 1;
    }
    let mut token = self.tokens[position];

    while position < self.tokens.len() - 1 && token.ttype == TokenType::Whitespace {
      position += 1;
      token = self.tokens[position];
    }

    if token.ttype == TokenType::Whitespace {
      TokenType::EndOfFile
    } else {
      token.ttype
    }
  }

  fn get(&self, position: usize) -> &'tokens Token {
    self.tokens.get(position).unwrap_or(&Token {
      ttype: TokenType::EndOfFile,
      line: 0,
      end: 0,
      start: 0,
    })
  }

  fn current(&self) -> &'tokens Token {
    self.get(self.position)
  }

  fn current_advance(&mut self) -> &'tokens Token {
    let token = self.current();
    self.next();
    token
  }

  fn expect(&mut self, token_type: TokenType, message: Error) -> Result<&'tokens Token, Error> {
    let current = self.current();
    if current.ttype == token_type {
      Ok(current)
    } else {
      Err(message)
    }
  }

  fn consume(&mut self, token_type: TokenType, message: Error) -> Result<&'tokens Token, Error> {
    let result = self.expect(token_type, message)?;
    self.next();
    Ok(result)
  }

  fn consume_next(
    &mut self,
    token_type: TokenType,
    message: Error,
  ) -> Result<&'tokens Token, Error> {
    self.next();
    self.consume(token_type, message)
  }

  fn expect_newline(&mut self) -> Result<(), Error> {
    if self.matches(TokenType::EndOfLine) {
      return Ok(());
    }

    let current = self.next();
    if current.ttype == TokenType::EndOfLine || current.ttype == TokenType::EndOfFile {
      self.next();
      Ok(())
    } else {
      Err(Error::ExpectedNewLine)
    }
  }

  fn ignore_newline(&mut self) {
    while self.matches(TokenType::EndOfLine) {}
  }

  fn matches(&mut self, token_type: TokenType) -> bool {
    let matches = self.current().ttype == token_type;
    if matches {
      self.next();
    }
    matches
  }

  fn skip_newline_if_illegal_line_start(&mut self) {
    if self.current().ttype == TokenType::EndOfLine && self.peek().is_illegal_line_start() {
      self.next();
    }
  }

  fn is_function_bracket(&self) -> bool {
    let mut position = self.position + 1;
    let mut token = self.get(position);
    let mut depth = 0;

    loop {
      if depth == 0 && token.ttype == TokenType::RightParen {
        position += 1;
        break;
      } else if token.ttype == TokenType::EndOfFile {
        return false;
      } else if token.ttype == TokenType::RightParen {
        depth += 1;
      } else if token.ttype == TokenType::LeftParen {
        depth -= 1;
      }

      position += 1;
      token = self.get(position);
    }

    while self.get(position).ttype == TokenType::Whitespace {
      position += 1;
    }

    token = self.get(position);
    token.ttype == TokenType::FatRightArrow || token.ttype == TokenType::RightArrow
  }

  fn parse_expression(&mut self, precedence: Precedence) -> ExpressionResult<'source> {
    self.ignore_newline();
    let token = self.current();

    let can_assign = precedence <= Precedence::Assignment;
    let prefix = self.prefix_rule(token.ttype, can_assign)?;
    let mut previous = vec![prefix];
    self.skip_newline_if_illegal_line_start();

    while precedence <= Precedence::from(self.current().ttype) {
      let token = self.current();
      let can_assign = precedence <= Precedence::Assignment;

      if let Some(value) = self.infix_rule(token.ttype, previous.pop().unwrap(), can_assign)? {
        previous.push(value);
      }

      self.skip_newline_if_illegal_line_start();
    }

    if can_assign && self.matches(TokenType::Equal) {
      Err(Error::InvalidAssignmentTarget)
    } else {
      Ok(previous.pop().unwrap())
    }
  }

  fn prefix_rule(&mut self, token_type: TokenType, can_assign: bool) -> ExpressionResult<'source> {
    match token_type {
      TokenType::LeftParen => self.grouping(),
      TokenType::Minus | TokenType::Bang => self.unary(),
      TokenType::Identifier => self.variable(can_assign),
      TokenType::Number
      | TokenType::String
      | TokenType::True
      | TokenType::False
      | TokenType::Null => self.literal(),
      TokenType::LeftSquare => self.list(),
      TokenType::Unknown => Err(Error::UnexpectedCharacter),
      _ => Err(Error::ExpectedExpression),
    }
  }

  fn infix_rule(
    &mut self,
    token_type: TokenType,
    previous: Expression<'source>,
    can_assign: bool,
  ) -> Result<Option<Expression<'source>>, Error> {
    match token_type {
      TokenType::LeftParen => Ok(Some(self.call(previous)?)),
      TokenType::LeftSquare => Ok(Some(self.index(previous, can_assign)?)),
      TokenType::Comment => Ok(Some(self.comment(previous))),
      TokenType::Plus
      | TokenType::Minus
      | TokenType::Star
      | TokenType::Slash
      | TokenType::BangEqual
      | TokenType::EqualEqual
      | TokenType::Greater
      | TokenType::GreaterEqual
      | TokenType::Less
      | TokenType::LessEqual
      | TokenType::And
      | TokenType::Or
      | TokenType::QuestionQuestion
      | TokenType::RightRight => Ok(Some(self.binary(previous)?)),
      _ => Ok(None),
    }
  }
}

// Statements
impl<'source> Parser<'source, '_> {
  fn block_depth(whitespace: &str) -> i32 {
    let mut depth = 0;
    for c in whitespace.chars() {
      match c {
        ' ' => depth += 1,
        '\t' => depth += 2,
        _ => {}
      }
    }
    depth /= 2;
    depth
  }

  fn statement(&mut self) -> StatementResult<'source> {
    self.ignore_newline();

    let last = self.position.saturating_sub(1);
    let mut last_token = self.get(last);

    let depth = Parser::block_depth(last_token.get_value(self.source));

    if last_token.ttype != TokenType::Whitespace || depth == 0 {
      return self.stmt();
    }

    let mut statements = Vec::new();

    while last_token.ttype == TokenType::Whitespace
      && Parser::block_depth(last_token.get_value(self.source)) >= depth
      && self.current().ttype != TokenType::EndOfFile
    {
      if Parser::block_depth(last_token.get_value(self.source)) > depth {
        statements.push(self.statement()?);
      } else {
        statements.push(self.stmt()?);
      }
      self.ignore_newline();
      last_token = self.get(self.position - 1);
    }

    Ok(statement!(
      Block { body: statements },
      (statements[0].span, statements.last().unwrap().span)
    ))
  }

  fn stmt(&mut self) -> StatementResult<'source> {
    let token = self.current();

    match token.ttype {
      TokenType::Let => self.var_declaration(),
      TokenType::If => self.if_statement(),
      TokenType::Return => self.return_statement(),
      TokenType::While => self.while_statement(),
      TokenType::EndOfFile => Err(Error::EmptyStatement),
      TokenType::Comment => self.comment_statement(),
      TokenType::From => self.import_statement(),
      _ => self.expression_statement(),
    }
  }

  fn var_declaration(&mut self) -> StatementResult<'source> {
    let token = self.current_advance();

    let (identifier, identifier_span) = match self.current().ttype {
      TokenType::Identifier => {
        let token = self.current_advance();
        (
          DeclarationIdentifier::Variable(token.get_value(self.source)),
          token.into(),
        )
      }
      TokenType::LeftSquare => {
        self.next();
        let mut identifiers = Vec::new();
        while self.current().ttype == TokenType::Identifier {
          identifiers.push(self.current_advance().get_value(self.source));
          self.matches(TokenType::Comma);
        }
        self.consume(TokenType::RightSquare, Error::ExpectedClosingSquare)?;

        (
          DeclarationIdentifier::List(identifiers),
          self.current().into(),
        )
      }
      _ => Err(Error::ExpectedIdentifier)?,
    };
    let type_ = if self.matches(TokenType::Colon) {
      Some(self.types()?)
    } else {
      None
    };

    let (end, expression) = if self.matches(TokenType::Equal) {
      let mut expression = self.expression()?;

      if let Expr::Function { ref mut name, .. } = expression.expr
        && let DeclarationIdentifier::Variable(identifier) = identifier
      {
        *name = Some(identifier);
      }

      (expression.span, Some(expression))
    } else {
      (identifier_span, None)
    };

    Ok(statement!(
      Declaration {
        identifier,
        type_,
        expression
      },
      (token, end)
    ))
  }

  fn return_statement(&mut self) -> StatementResult<'source> {
    let token = self.current_advance();
    if self.matches(TokenType::EndOfLine) {
      Ok(statement!(Return { expression: None }, token))
    } else {
      let expression = self.expression()?;
      self.expect_newline()?;

      Ok(statement!(
        Return {
          expression: Some(expression)
        },
        (token, expression.span)
      ))
    }
  }

  fn if_statement(&mut self) -> StatementResult<'source> {
    let if_token = self.current();
    self.consume_next(TokenType::LeftParen, Error::ExpectedOpeningBracket)?;
    let condition = self.expression()?;
    self.ignore_newline();
    self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;

    self.matches(TokenType::EndOfLine);
    let body = self.statement()?;

    self.ignore_newline();

    let (end, otherwise) = if self.matches(TokenType::Else) {
      self.ignore_newline();
      let statement = self.statement()?;

      (statement.span, Some(Box::new(statement)))
    } else {
      (body.span, None)
    };

    Ok(statement!(
      If {
        condition,
        then: Box::new(body),
        otherwise
      },
      (if_token, end)
    ))
  }

  fn while_statement(&mut self) -> StatementResult<'source> {
    let token = self.current();
    self.consume_next(TokenType::LeftParen, Error::ExpectedOpeningBracket)?;
    let condition = self.expression()?;
    self.ignore_newline();
    self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;
    self.matches(TokenType::EndOfLine);

    let body = self.statement()?;

    Ok(statement!(
      While {
        condition,
        body: Box::new(body)
      },
      (token, body.span)
    ))
  }

  fn comment_statement(&mut self) -> StatementResult<'source> {
    let token = self.current_advance();
    let text = token.get_value(self.source);
    self.expect_newline()?;

    Ok(statement!(Comment { text }, token))
  }

  fn expression_statement(&mut self) -> StatementResult<'source> {
    let expression = self.expression()?;
    self.expect_newline()?;

    Ok(statement!(Expression { expression }, expression.span))
  }

  fn import_statement(&mut self) -> StatementResult<'source> {
    let token = self.current_advance();
    let module = self.consume(TokenType::Identifier, Error::ExpectedIdentifier)?;
    self.consume(TokenType::Import, Error::ExpectedImportKeyword)?;
    self.consume(TokenType::LeftBrace, Error::ExpectedOpeningBrace)?;

    let mut items = Vec::new();
    let end_token = loop {
      self.ignore_newline();
      if self.current().ttype == TokenType::RightBrace {
        break self.current();
      }

      let item = self.consume(TokenType::Identifier, Error::ExpectedIdentifier)?;
      let alias = if self.matches(TokenType::As) {
        let token = self.consume(TokenType::Identifier, Error::ExpectedIdentifier)?;
        Some(token.get_value(self.source))
      } else {
        None
      };

      items.push(ImportItem {
        name: item.get_value(self.source),
        span: Span::from(item),
        alias,
      });

      if !self.matches(TokenType::Comma) {
        self.ignore_newline();
        break self.expect(TokenType::RightBrace, Error::ExpectedClosingBrace)?;
      }
    };

    self.expect_newline()?;

    Ok(statement!(
      Import {
        module: module.get_value(self.source),
        items,
      },
      (token, end_token)
    ))
  }
}

// Expressions
impl<'source> Parser<'source, '_> {
  fn expression(&mut self) -> ExpressionResult<'source> {
    self.parse_expression(Precedence::Assignment)
  }

  fn function(&mut self) -> ExpressionResult<'source> {
    let token = self.current_advance();

    let mut parameters = Vec::new();
    loop {
      self.ignore_newline();
      if self.matches(TokenType::RightParen) {
        break;
      }

      let catch_remaining = self.matches(TokenType::DotDot);
      let parameter = self.consume(TokenType::Identifier, Error::ExpectedIdentifier)?;
      let type_ = if self.matches(TokenType::Colon) {
        Some(self.types()?)
      } else {
        None
      };

      parameters.push(Parameter {
        name: parameter.get_value(self.source),
        span: Span::from(parameter),
        type_,
        catch_remaining,
      });

      if !self.matches(TokenType::Comma) {
        self.ignore_newline();
        self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;
        break;
      } else if catch_remaining {
        self.consume(TokenType::RightParen, Error::ExpectedCatchAllLast)?;
        break;
      }
    }

    let (body, return_type) = if self.matches(TokenType::FatRightArrow) {
      let expression = self.expression()?;

      (
        Ok(statement!(
          Return {
            expression: Some(expression)
          },
          (token, expression.span)
        )),
        None,
      )
    } else if self.matches(TokenType::RightArrow) {
      let return_type = self.optional_types()?;
      self.ignore_newline();
      let statement = self.statement()?;

      (Ok(statement), return_type)
    } else {
      (Err(Error::ExpectedFunctionArrow), None)
    };
    let body = body?;

    Ok(expression!(
      Function {
        body: Box::new(body),
        parameters,
        name: None,
        return_type
      },
      (token, body.span)
    ))
  }

  fn grouping(&mut self) -> ExpressionResult<'source> {
    if self.is_function_bracket() {
      return self.function();
    }

    let token = self.current_advance();
    let expression = self.expression()?;
    self.ignore_newline();
    let end_token = self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;

    Ok(expression!(
      Group {
        expression: Box::new(expression),
      },
      (token, end_token)
    ))
  }

  fn unary(&mut self) -> ExpressionResult<'source> {
    let token = self.current_advance();
    let expression = self.parse_expression(Precedence::Unary)?;

    Ok(expression!(
      Unary {
        operator: UnaryOperator::from(token.ttype),
        expression: Box::new(expression),
      },
      (token, expression.span)
    ))
  }

  fn literal(&mut self) -> ExpressionResult<'source> {
    let token = self.current_advance();
    let string = token.get_value(self.source);

    let value = if token.ttype != TokenType::String {
      Ok(string)
    } else if string[0..1] == string[string.len() - 1..string.len()] {
      Ok(&string[1..string.len() - 1])
    } else {
      Err(Error::UnterminatedString)
    }?;

    Ok(expression!(
      Literal {
        value,
        type_: LiteralType::from(token.ttype)
      },
      token
    ))
  }

  fn list(&mut self) -> ExpressionResult<'source> {
    let start_token = self.current_advance();

    let mut items = Vec::new();
    let end_token = loop {
      self.ignore_newline();
      if self.current().ttype == TokenType::RightSquare {
        break self.current_advance();
      }

      items.push(self.expression()?);

      if !self.matches(TokenType::Comma) {
        self.ignore_newline();
        break self.consume(TokenType::RightSquare, Error::ExpectedClosingSquare)?;
      }
    };

    Ok(expression!(List { items }, (start_token, end_token)))
  }

  fn variable(&mut self, can_assign: bool) -> ExpressionResult<'source> {
    let identifier = self.current_advance();
    let name = identifier.get_value(self.source);
    let is_assignment_operator = self.current().ttype.is_assignment_operator();

    if (true, true) == (can_assign, is_assignment_operator) {
      let operator = self.current_advance();
      let right = self.expression()?;
      let end_span = right.span;

      let binary = expression!(
        Binary {
          operator: AssignmentOperator::token_to_binary(operator.ttype),
          left: Box::new(expression!(Variable { name }, identifier)),
          right: Box::new(right),
        },
        (identifier, end_span)
      );

      Ok(expression!(
        Assignment {
          identifier: name,
          expression: Box::new(binary)
        },
        (identifier, end_span)
      ))
    } else if self.matches(TokenType::Equal) && can_assign {
      let expression = self.expression()?;

      Ok(expression!(
        Assignment {
          identifier: name,
          expression: Box::new(expression),
        },
        (identifier, expression.span)
      ))
    } else {
      Ok(expression!(
        Variable {
          name: identifier.get_value(self.source)
        },
        identifier
      ))
    }
  }

  fn index(
    &mut self,
    previous: Expression<'source>,
    can_assign: bool,
  ) -> ExpressionResult<'source> {
    let _start_token = self.current_advance();
    self.ignore_newline();
    let expression = self.expression()?;
    self.ignore_newline();
    let end_token = self.consume(TokenType::RightSquare, Error::ExpectedClosingSquare)?;

    let is_assignment_operator = self.current().ttype.is_assignment_operator();

    if (true, true) == (can_assign, is_assignment_operator) {
      let operator = self.current_advance();
      let right = self.expression()?;

      Ok(expression!(
        IndexAssignment {
          expression: Box::new(previous),
          index: Box::new(expression),
          assignment_operator: AssignmentOperator::from_token(operator.ttype),
          value: Box::new(right)
        },
        (previous.span, right.span)
      ))
    } else if self.matches(TokenType::Equal) && can_assign {
      let right = self.expression()?;

      Ok(expression!(
        IndexAssignment {
          expression: Box::new(previous),
          index: Box::new(expression),
          assignment_operator: None,
          value: Box::new(right)
        },
        (previous.span, right.span)
      ))
    } else {
      Ok(expression!(
        Index {
          expression: Box::new(previous),
          index: Box::new(expression),
        },
        (previous.span, end_token)
      ))
    }
  }

  fn call(&mut self, previous: Expression<'source>) -> ExpressionResult<'source> {
    let _start_token = self.current_advance();
    let mut arguments = Vec::new();
    let end_token = loop {
      self.ignore_newline();
      if self.current().ttype == TokenType::RightParen {
        break self.current_advance();
      }

      arguments.push(self.expression()?);

      if !self.matches(TokenType::Comma) {
        self.ignore_newline();
        break self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;
      }
    };

    Ok(expression!(
      Call {
        expression: Box::new(previous),
        arguments,
      },
      (previous.span, end_token)
    ))
  }

  fn comment(&mut self, previous: Expression<'source>) -> Expression<'source> {
    let token = self.current_advance();

    expression!(
      Comment {
        expression: Box::new(previous),
        text: token.get_value(self.source),
      },
      (previous.span, token)
    )
  }

  fn binary(&mut self, previous: Expression<'source>) -> ExpressionResult<'source> {
    let operator = self.current_advance();
    let precedence = Precedence::from(operator.ttype);
    let right = self.parse_expression(precedence.next())?;

    Ok(expression!(
      Binary {
        operator: BinaryOperator::from(operator.ttype),
        left: Box::new(previous),
        right: Box::new(right),
      },
      (previous.span, right.span)
    ))
  }
}

// Types
impl<'source> Parser<'source, '_> {
  fn optional_types(&mut self) -> Result<Option<TypeExpression<'source>>, Error> {
    if matches!(
      self.current().ttype,
      TokenType::EndOfLine | TokenType::Comma
    ) {
      Ok(None)
    } else {
      Ok(Some(self.types()?))
    }
  }

  fn types(&mut self) -> TypeResult<'source> {
    let token = self.current();

    let mut t = match token.ttype {
      TokenType::Identifier | TokenType::Null | TokenType::True | TokenType::False => {
        self.next();
        Ok(types!(Named(token.get_value(self.source)), token))
      }
      TokenType::LeftParen => self.type_group(),
      _ => Err(Error::ExpectedType),
    };

    if self.matches(TokenType::LeftSquare) {
      t = self.type_list(t?);
    }

    match self.current().ttype {
      TokenType::Pipe => self.type_union(t?),
      TokenType::Question => Ok(self.type_optional(t?)),
      _ => t,
    }
  }

  fn type_union(&mut self, left: TypeExpression<'source>) -> TypeResult<'source> {
    let _token = self.current_advance();
    let right = self.types()?;

    Ok(types!(
      Union(Box::new(left), Box::new(right)),
      (left.span, right.span)
    ))
  }

  fn type_optional(&mut self, left: TypeExpression<'source>) -> TypeExpression<'source> {
    let token = self.current_advance();

    types!(Optional(Box::new(left)), (left.span, token))
  }

  fn type_list(&mut self, left: TypeExpression<'source>) -> TypeResult<'source> {
    let end_token = self.consume(TokenType::RightSquare, Error::ExpectedClosingSquare)?;

    Ok(types!(List(Box::new(left)), (left.span, end_token)))
  }

  fn type_group(&mut self) -> TypeResult<'source> {
    if self.is_function_bracket() {
      return self.type_function();
    }

    let token = self.current_advance();
    let types = self.types()?;
    let end_token = self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;

    Ok(types!(Group(Box::new(types)), (token, end_token)))
  }

  fn type_function(&mut self) -> TypeResult<'source> {
    let start_token = *self.next();
    let mut parameters = Vec::new();
    loop {
      if self.matches(TokenType::RightParen) {
        break;
      }

      parameters.push(self.types()?);

      if !self.matches(TokenType::Comma) {
        self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;
        break;
      }
    }

    self.consume(TokenType::RightArrow, Error::ExpectedFunctionArrow)?;
    let return_type = self.types()?;

    Ok(types!(
      Function(Box::new(return_type), parameters),
      (start_token, self.current())
    ))
  }
}

pub fn parse(source: &str) -> Result<Vec<Statement>, Diagnostic> {
  let tokens = tokenize(source);
  let mut parser = Parser::new(source, &tokens);
  let mut statements = Vec::new();

  while !parser.at_end() {
    match parser.statement() {
      Ok(stmt) => statements.push(stmt),
      Err(Error::EmptyStatement) => {}
      Err(err) => {
        let last_token = if parser.current().ttype == TokenType::EndOfFile {
          parser.tokens[parser.tokens.len().saturating_sub(1)]
        } else {
          *parser.current()
        };
        return Err(err.get_diagnostic(source, &last_token));
      }
    }
  }

  Ok(statements)
}

pub fn parse_number(string: &str) -> f64 {
  string
    .replace('_', "")
    .parse()
    .expect("String to be valid number representation")
}

#[cfg(test)]
mod tests {
  use super::*;

  fn assert_literal(expr: &Expr<'_>, expected: &str, literal_type: LiteralType) {
    match expr {
      Expr::Literal { value, type_, .. } => {
        assert_eq!(*value, expected);
        assert_eq!(type_, &literal_type);
      }
      _ => panic!("Expected literal"),
    }
  }

  fn assert_variable(expr: &Expr<'_>, expected: &str) {
    match expr {
      Expr::Variable { name } => {
        assert_eq!(name, &expected);
      }
      _ => panic!("Expected literal"),
    }
  }

  fn unwrap_expression<'s>(statement: &'s Statement<'s>) -> &'s Expr<'s> {
    if let Stmt::Expression { expression } = &statement.stmt {
      &expression.expr
    } else {
      panic!("Expected expression");
    }
  }

  #[test]
  fn should_error_on_unknown_character() {
    let result = super::parse("&");

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message, "Unknown character '&'");
  }

  #[test]
  fn should_parse_group() {
    let statements = super::parse("('hello world')\n").unwrap();

    if let Expr::Group { expression, .. } = unwrap_expression(&statements[0]) {
      assert_literal(expression, "hello world", LiteralType::String);
    } else {
      panic!("Expected group expression statement");
    }
  }

  #[test]
  fn should_parse_unary() {
    let statements = super::parse("!false\n").unwrap();

    if let Expr::Unary {
      operator,
      expression,
    } = unwrap_expression(&statements[0])
    {
      assert_literal(expression, "false", LiteralType::False);
      assert_eq!(*operator, UnaryOperator::Not);
    } else {
      panic!("Expected unary expression statement");
    }
  }

  #[test]
  fn should_parse_binary() {
    let statements = super::parse("10 + 5\n").unwrap();

    if let Expr::Binary {
      operator,
      left,
      right,
    } = unwrap_expression(&statements[0])
    {
      assert_literal(left, "10", LiteralType::Number);
      assert_literal(right, "5", LiteralType::Number);
      assert_eq!(*operator, BinaryOperator::Plus);
    } else {
      panic!("Expected binary expression statement");
    }
  }

  #[test]
  fn should_parse_call() {
    let statements = super::parse("function(7, null)\n").unwrap();

    if let Expr::Call {
      expression,
      arguments,
      ..
    } = unwrap_expression(&statements[0])
    {
      assert_literal(&arguments[0], "7", LiteralType::Number);
      assert_literal(&arguments[1], "null", LiteralType::Null);
      assert_variable(expression, "function");
    } else {
      panic!("Expected binary expression statement");
    }
  }

  #[test]
  fn should_parse_function() {
    let statements = super::parse("() => null\n").unwrap();

    if let Expr::Function {
      parameters, body, ..
    } = unwrap_expression(&statements[0])
    {
      assert_eq!(parameters.len(), 0);

      if let Stmt::Return {
        expression: Some(expression),
        ..
      } = &body.stmt
      {
        assert_literal(expression, "null", LiteralType::Null);
      }
    } else {
      panic!("Expected return statement");
    }
  }

  #[test]
  fn should_parse_variable_declaration_with_initalizer() {
    let statements = super::parse("let a = null\n").unwrap();

    if let Stmt::Declaration {
      identifier,
      expression,
      ..
    } = &statements[0].stmt
    {
      let expr = expression.as_ref().unwrap();
      match identifier {
        DeclarationIdentifier::Variable(identifier) => assert_eq!(*identifier, "a"),
        _ => assert!(false),
      };
      assert_literal(&expr.expr, "null", LiteralType::Null);
    } else {
      panic!("Expected declaration statement");
    }
  }

  #[test]
  fn should_parse_variable_declaration_list_destructuring() {
    let statements = super::parse("let [a, b] = null\n").unwrap();

    if let Stmt::Declaration {
      identifier,
      expression,
      ..
    } = &statements[0].stmt
    {
      let expr = expression.as_ref().unwrap();
      match identifier {
        DeclarationIdentifier::List(identifier) => assert_eq!(identifier.len(), 2),
        _ => assert!(false),
      };
      assert_literal(&expr.expr, "null", LiteralType::Null);
    } else {
      panic!("Expected declaration statement");
    }
  }

  #[test]
  fn should_parse_variable_declaration_without_initalizer() {
    let statements = super::parse("let b\n").unwrap();

    if let Stmt::Declaration {
      identifier,
      expression: None,
      ..
    } = &statements[0].stmt
    {
      match identifier {
        DeclarationIdentifier::Variable(identifier) => assert_eq!(*identifier, "b"),
        _ => assert!(false),
      };
    } else {
      panic!("Expected declaration statement");
    }
  }

  #[test]
  fn should_parse_return_with_value() {
    let statements = super::parse("return value\n").unwrap();

    if let Stmt::Return {
      expression: Some(expression),
      ..
    } = &statements[0].stmt
    {
      assert_variable(expression, "value");
    } else {
      panic!("Expected return statement");
    }
  }

  #[test]
  fn should_parse_return_without_value() {
    let statements = super::parse("return\n").unwrap();

    if let Stmt::Return {
      expression: Some(_),
    } = &statements[0].stmt
    {
      panic!("Expected return statement");
    }
  }

  #[test]
  fn should_parse_while() {
    let statements = super::parse("while(7) doStuff\n").unwrap();

    if let Stmt::While {
      condition, body, ..
    } = &statements[0].stmt
    {
      assert_literal(&condition.expr, "7", LiteralType::Number);
      assert_variable(unwrap_expression(body), "doStuff");
    } else {
      panic!("Expected while statement");
    }
  }

  #[test]
  fn should_parse_if_else() {
    let statements = super::parse("if (true) doStuff\n").unwrap();

    if let Stmt::If {
      condition,
      then,
      otherwise,
      ..
    } = &statements[0].stmt
    {
      assert_literal(condition, "true", LiteralType::True);
      assert_variable(unwrap_expression(then), "doStuff");
      assert!(otherwise.is_none());
    } else {
      panic!("Expected if statement");
    }
  }

  #[test]
  fn should_parse_if_with_else() {
    let statements = super::parse("if (true)\n\tdoStuff\nelse\n\tdoOtherStuff\n").unwrap();

    if let Stmt::If {
      condition,
      otherwise,
      ..
    } = &statements[0].stmt
    {
      assert_literal(condition, "true", LiteralType::True);
      assert!(otherwise.is_some());
    } else {
      panic!("Expected if statement");
    }
  }

  #[test]
  fn should_parse_block() {
    let statements = super::parse("a\n\tdoStuff\n\totherStuff\n\tmoreStuff\n").unwrap();

    assert_eq!(statements.len(), 2);

    if let Stmt::Block { body } = &statements[1].stmt {
      assert_eq!(body.len(), 3);
    } else {
      panic!("Expected block statement");
    }
  }

  #[test]
  fn should_parse_list() {
    let statements = super::parse("[44, null, 'hello']\n").unwrap();

    if let Expr::List { items } = unwrap_expression(&statements[0]) {
      assert_literal(&items[0], "44", LiteralType::Number);
      assert_literal(&items[1], "null", LiteralType::Null);
      assert_literal(&items[2], "hello", LiteralType::String);
    } else {
      panic!("Expected list");
    }
  }

  #[test]
  fn should_parse_index() {
    let statements = super::parse("a[5]\n").unwrap();

    if let Expr::Index { expression, index } = unwrap_expression(&statements[0]) {
      assert_literal(&index, "5", LiteralType::Number);
      assert_variable(expression, "a");
    } else {
      panic!("Expected list");
    }
  }
}
