use crate::{
  ast::{
    expression::{expression, operators, Expr, Expression, LiteralType, Parameter},
    statement::{statement, AliasItem, DeclarationIdentifier, Statement, Stmt},
    types::{types, Type, TypeExpression},
  },
  tokens::{Token, TokenType, Tokeniser},
  LineNumber, Span,
};
use std::{error, fmt, iter, str};

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
      TokenType::Star | TokenType::Slash | TokenType::Percent => Self::Factor,
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
  ExpectedClosingAngle,
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
  ExpectedModuleItem,
  ExpectedColon,
}
impl Error {
  fn get_title(&self) -> &'static str {
    match self {
      Self::ExpectedOpeningBracket => "Expected '('",
      Self::ExpectedClosingBracket => "Expected ')'",
      Self::ExpectedOpeningBrace => "Expected '{'",
      Self::ExpectedClosingBrace => "Expected '}'",
      Self::ExpectedClosingSquare => "Expected ']'",
      Self::ExpectedClosingAngle => "Expected '>'",
      Self::ExpectedColon => "Expected ':'",
      Self::ExpectedExpression => "Expected Expression",
      Self::ExpectedFunctionArrow => "Expected Function Arrow (-> / =>)",
      Self::ExpectedNewLine => "Expected New Line",
      Self::ExpectedIdentifier => "Expected Identifier",
      Self::InvalidAssignmentTarget => "Invalid Assignment Target",
      Self::UnexpectedCharacter => "Unexpected Character",
      Self::UnterminatedString => "Unterminated String",
      Self::ExpectedImportKeyword => "Expected 'import' keyword",
      Self::ExpectedType => "Expected Type",
      Self::ExpectedModuleItem => "Expected Module Item to Import",
      Self::EmptyStatement => unreachable!("EmptyStatement caught to return nothing"),
    }
  }

  fn get_message(&self, source: &[u8], token: Token) -> String {
    match self {
      Self::ExpectedOpeningBracket
      | Self::ExpectedClosingBracket
      | Self::ExpectedOpeningBrace
      | Self::ExpectedClosingBrace
      | Self::ExpectedClosingSquare
      | Self::ExpectedClosingAngle
      | Self::ExpectedExpression
      | Self::ExpectedFunctionArrow
      | Self::ExpectedNewLine
      | Self::ExpectedIdentifier
      | Self::ExpectedImportKeyword
      | Self::ExpectedModuleItem
      | Self::ExpectedColon
      | Self::ExpectedType => format!("but recieved '{}'", token.get_value(source)),
      Self::UnexpectedCharacter => format!("Unknown character '{}'", token.get_value(source)),
      Self::UnterminatedString => {
        format!("Missing closing quote {}", &token.get_value(source)[0..1])
      }
      Self::InvalidAssignmentTarget => "Can't assign to an expression, only a variable".to_string(),
      Self::EmptyStatement => unreachable!("EmptyStatement caught to return nothing"),
    }
  }

  fn get_diagnostic(&self, source: &str, token: Token) -> Diagnostic {
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

pub struct Parser<'source> {
  source: &'source [u8],
  tokeniser: iter::Peekable<Tokeniser<'source>>,

  current: Token,
  previous: Token,
}

impl<'source> Parser<'source> {
  pub fn new(source: &'source str) -> Self {
    let mut tokeniser = Tokeniser::new(source).peekable();
    let current = tokeniser.next().unwrap_or_default();

    Self {
      source: source.as_bytes(),
      tokeniser,

      current,
      previous: Token::default(),
    }
  }

  pub fn number(string: &str) -> f64 {
    string
      .replace('_', "")
      .parse()
      .expect("String to be valid number representation")
  }

  fn at_end(&mut self) -> bool {
    self.current.ttype == TokenType::EndOfFile
  }

  fn next(&mut self) -> Token {
    let token = self.tokeniser.next().unwrap_or_default();

    self.previous = self.current;
    self.current = token;

    if token.ttype == TokenType::Whitespace {
      self.next()
    } else {
      token
    }
  }

  fn peek(&mut self) -> TokenType {
    self.tokeniser.peek().copied().unwrap_or_default().ttype
  }

  fn current_advance(&mut self) -> Token {
    self.next();
    self.previous
  }

  fn expect(&mut self, token_type: TokenType, message: Error) -> Result<Token, Error> {
    if self.current.ttype == token_type {
      Ok(self.current)
    } else {
      Err(message)
    }
  }

  fn consume(&mut self, token_type: TokenType, message: Error) -> Result<Token, Error> {
    let result = self.expect(token_type, message)?;
    self.next();
    Ok(result)
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
    let matches = self.current.ttype == token_type;
    if matches {
      self.next();
    }
    matches
  }

  fn parse_expression(&mut self, precedence: Precedence) -> ExpressionResult<'source> {
    self.ignore_newline();

    let can_assign = precedence <= Precedence::Assignment;
    let prefix = self.prefix_rule(self.current.ttype, can_assign)?;

    let mut previous = prefix;
    while precedence <= Precedence::from(self.current.ttype) {
      let can_assign = precedence <= Precedence::Assignment;
      previous = self.infix_rule(self.current.ttype, previous, can_assign)?;
    }

    if can_assign && self.matches(TokenType::Equal) {
      Err(Error::InvalidAssignmentTarget)
    } else {
      Ok(previous)
    }
  }

  fn prefix_rule(&mut self, token_type: TokenType, can_assign: bool) -> ExpressionResult<'source> {
    match token_type {
      TokenType::LeftParen => self.grouping_or_function(),
      TokenType::Minus | TokenType::Bang => self.unary(),
      TokenType::Identifier => self.variable(can_assign),
      TokenType::Number
      | TokenType::String
      | TokenType::True
      | TokenType::False
      | TokenType::Null => self.literal(),
      TokenType::FormatStringStart => self.format_string(),
      TokenType::LeftSquare => self.list(),
      TokenType::LeftBrace => self.dictionary(),
      TokenType::Unknown => Err(Error::UnexpectedCharacter),
      _ => Err(Error::ExpectedExpression),
    }
  }

  fn infix_rule(
    &mut self,
    token_type: TokenType,
    previous: Expression<'source>,
    can_assign: bool,
  ) -> Result<Expression<'source>, Error> {
    match token_type {
      TokenType::LeftParen => Ok(self.call(previous)?),
      TokenType::LeftSquare => Ok(self.index(previous, can_assign)?),
      TokenType::Comment => Ok(self.comment(previous)),
      TokenType::Plus
      | TokenType::Minus
      | TokenType::Star
      | TokenType::Slash
      | TokenType::Percent
      | TokenType::BangEqual
      | TokenType::EqualEqual
      | TokenType::Greater
      | TokenType::GreaterEqual
      | TokenType::Less
      | TokenType::LessEqual
      | TokenType::And
      | TokenType::Or
      | TokenType::QuestionQuestion
      | TokenType::RightRight => Ok(self.binary(previous)?),
      _ => unreachable!(),
    }
  }
}

// Statements
impl<'source> Parser<'source> {
  fn block_depth(&self, token: Token) -> i32 {
    let mut depth = 0;
    for c in token.get_value(self.source).chars() {
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

    let mut last_token = self.previous;
    let depth = self.block_depth(last_token);

    if last_token.ttype != TokenType::Whitespace || depth == 0 {
      return self.stmt();
    }

    let mut statements = Vec::new();

    while last_token.ttype == TokenType::Whitespace
      && self.block_depth(last_token) >= depth
      && self.current.ttype != TokenType::EndOfFile
    {
      if self.block_depth(last_token) > depth {
        statements.push(self.statement()?);
      } else {
        statements.push(self.stmt()?);
      }
      self.ignore_newline();
      last_token = self.previous;
    }

    if statements.is_empty() {
      Err(Error::EmptyStatement)?;
    }

    Ok(statement!(
      Block { body: statements },
      (statements[0].span, statements.last().unwrap().span)
    ))
  }

  fn stmt(&mut self) -> StatementResult<'source> {
    match self.current.ttype {
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

    let (identifier, identifier_end) = match self.current.ttype {
      TokenType::Identifier => {
        let token = self.current_advance();
        (
          DeclarationIdentifier::Variable(token.get_value(self.source)),
          token,
        )
      }
      TokenType::LeftSquare => {
        self.next();
        let mut identifiers = Vec::new();
        while self.current.ttype == TokenType::Identifier {
          identifiers.push(self.current_advance().get_value(self.source));
          self.matches(TokenType::Comma);
          self.ignore_newline();
        }
        self.consume(TokenType::RightSquare, Error::ExpectedClosingSquare)?;

        (DeclarationIdentifier::Ordered(identifiers), self.current)
      }
      TokenType::LeftBrace => {
        self.next();
        let identifiers = self.alias_items()?;
        let end_token = self.current_advance();

        (DeclarationIdentifier::Named(identifiers), end_token)
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
      (identifier_end.into(), None)
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
    let if_token = self.current_advance();
    self.consume(TokenType::LeftParen, Error::ExpectedOpeningBracket)?;
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
    let token = self.current_advance();
    self.consume(TokenType::LeftParen, Error::ExpectedOpeningBracket)?;
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

  fn alias_items(&mut self) -> Result<Vec<AliasItem<'source>>, Error> {
    let mut items = Vec::new();

    loop {
      self.ignore_newline();
      if self.current.ttype == TokenType::RightBrace {
        break;
      }

      let item = self.consume(TokenType::Identifier, Error::ExpectedIdentifier)?;
      let alias = if self.matches(TokenType::As) {
        let token = self.consume(TokenType::Identifier, Error::ExpectedIdentifier)?;
        Some(token.get_value(self.source))
      } else {
        None
      };

      items.push(AliasItem {
        name: item.get_value(self.source),
        span: Span::from(item),
        alias,
      });

      if !self.matches(TokenType::Comma) {
        self.ignore_newline();
        self.expect(TokenType::RightBrace, Error::ExpectedClosingBrace)?;
        break;
      }
    }

    Ok(items)
  }

  fn import_statement(&mut self) -> StatementResult<'source> {
    let token = self.current_advance();
    let module = if self.current.ttype == TokenType::String {
      let string = self.current_advance().get_value(self.source);
      Self::check_string(string)?
    } else {
      self
        .consume(TokenType::Identifier, Error::ExpectedIdentifier)?
        .get_value(self.source)
    };

    self.consume(TokenType::Import, Error::ExpectedImportKeyword)?;
    self.consume(TokenType::LeftBrace, Error::ExpectedOpeningBrace)?;

    let items = self.alias_items()?;
    let end_token = self.current;

    self.expect_newline()?;

    Ok(statement!(Import { module, items }, (token, end_token)))
  }
}

// Expressions
impl<'source> Parser<'source> {
  fn expression(&mut self) -> ExpressionResult<'source> {
    self.parse_expression(Precedence::Assignment)
  }

  fn grouping_or_function(&mut self) -> ExpressionResult<'source> {
    let opening_bracket = self.current_advance();
    self.ignore_newline();

    match self.current.ttype {
      TokenType::Identifier => match self.peek() {
        TokenType::Colon | TokenType::Comma => self.function(opening_bracket),
        TokenType::RightParen => {
          let identifier = self.current_advance();
          let closing_bracket = self.current_advance();

          match self.current.ttype {
            TokenType::RightArrow | TokenType::FatRightArrow => {
              let parameter = Parameter {
                name: identifier.get_value(self.source),
                span: identifier.into(),
                type_: None,
              };
              self.function_body(opening_bracket, vec![parameter])
            }
            _ => {
              let identifier = expression!(
                Variable {
                  name: identifier.get_value(self.source)
                },
                identifier
              );
              Ok(expression!(
                Group {
                  expression: identifier.into()
                },
                (opening_bracket, closing_bracket)
              ))
            }
          }
        }
        _ => self.grouping(opening_bracket),
      },
      TokenType::RightParen => self.function(opening_bracket),
      _ => self.grouping(opening_bracket),
    }
  }

  fn function(&mut self, opening_bracket: Token) -> ExpressionResult<'source> {
    let mut parameters = Vec::new();
    loop {
      self.ignore_newline();
      if self.matches(TokenType::RightParen) {
        break;
      }

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
      });

      if !self.matches(TokenType::Comma) {
        self.ignore_newline();
        self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;
        break;
      }
    }

    self.function_body(opening_bracket, parameters)
  }

  fn function_body(
    &mut self,
    opening_bracket: Token,
    parameters: Vec<Parameter<'source>>,
  ) -> ExpressionResult<'source> {
    let (body, return_type) = if self.matches(TokenType::FatRightArrow) {
      let expression = self.expression()?;

      (
        Ok(statement!(
          Return {
            expression: Some(expression)
          },
          (opening_bracket, expression.span)
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
      (opening_bracket, body.span)
    ))
  }

  fn grouping(&mut self, opening_bracket: Token) -> ExpressionResult<'source> {
    let expression = self.expression()?;
    self.ignore_newline();
    let end_token = self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;

    Ok(expression!(
      Group {
        expression: Box::new(expression),
      },
      (opening_bracket, end_token)
    ))
  }

  fn unary(&mut self) -> ExpressionResult<'source> {
    let token = self.current_advance();
    let expression = self.parse_expression(Precedence::Unary)?;

    Ok(expression!(
      Unary {
        operator: operators::Unary::from(token.ttype),
        expression: Box::new(expression),
      },
      (token, expression.span)
    ))
  }

  fn literal(&mut self) -> ExpressionResult<'source> {
    let token = self.current_advance();
    let string = token.get_value(self.source);

    let value = if token.ttype == TokenType::String {
      Self::check_string(string)
    } else {
      Ok(string)
    }?;

    Ok(expression!(
      Literal {
        value,
        type_: LiteralType::from(token.ttype)
      },
      token
    ))
  }

  fn check_string(string: &str) -> Result<&str, Error> {
    if string.chars().next() == string.chars().last() && string.len() > 1 {
      Ok(&string[1..string.len() - 1])
    } else {
      Err(Error::UnterminatedString)
    }
  }

  fn format_string(&mut self) -> ExpressionResult<'source> {
    let token = self.current_advance();
    let start = token.get_value(self.source);
    let quote = start.as_bytes()[0] as char;

    let mut strings = vec![start[1..start.len() - 2].into()];
    let mut expressions = Vec::new();

    let end_token = loop {
      expressions.push(self.expression()?);

      match self.current.ttype {
        TokenType::FormatStringPart => {
          let part = self.current_advance().get_value(self.source);
          strings.push(part[1..part.len() - 2].into());
        }
        TokenType::FormatStringEnd => {
          let part = self.current.get_value(self.source);
          if !part.ends_with(quote) {
            Err(Error::UnterminatedString)?;
          }
          strings.push(part[1..part.len() - 1].into());
          break self.current_advance();
        }
        _ => Err(Error::UnterminatedString)?,
      }
    };

    Ok(expression!(
      FormatString {
        expressions,
        strings
      },
      (token, end_token)
    ))
  }

  fn list(&mut self) -> ExpressionResult<'source> {
    let start_token = self.current_advance();

    let mut items = Vec::new();
    let end_token = loop {
      self.ignore_newline();
      if self.current.ttype == TokenType::RightSquare {
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

  fn dictionary(&mut self) -> ExpressionResult<'source> {
    let start_token = self.current_advance();

    let mut items = Vec::new();
    let end_token = loop {
      self.ignore_newline();
      if self.current.ttype == TokenType::RightBrace {
        break self.current_advance();
      }

      if self.current.ttype == TokenType::Identifier
        && (self.peek() == TokenType::Comma || self.peek() == TokenType::RightBrace)
      {
        let name = self.current.get_value(self.source);
        let key = expression!(
          Literal {
            type_: LiteralType::String,
            value: name
          },
          self.current
        );
        let value = expression!(Variable { name }, self.current);

        items.push((key, value));
        self.next();
      } else {
        let key = self.expression()?;
        self.consume(TokenType::Colon, Error::ExpectedColon)?;
        self.ignore_newline();
        let value = self.expression()?;
        items.push((key, value));
      }

      if !self.matches(TokenType::Comma) {
        self.ignore_newline();
        break self.consume(TokenType::RightBrace, Error::ExpectedClosingSquare)?;
      }
    };

    Ok(expression!(Dictionary { items }, (start_token, end_token)))
  }

  fn variable(&mut self, can_assign: bool) -> ExpressionResult<'source> {
    let identifier = self.current_advance();
    let name = identifier.get_value(self.source);
    let is_assignment_operator = self.current.ttype.is_assignment_operator();

    if (true, true) == (can_assign, is_assignment_operator) {
      let operator = self.current_advance();
      let right = self.expression()?;
      let end_span = right.span;

      let binary = expression!(
        Binary {
          operator: operators::Assignment::token_to_binary(operator.ttype),
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
    } else if self.matches(TokenType::ColonColon) {
      let item = self.consume(TokenType::Identifier, Error::ExpectedModuleItem)?;

      Ok(expression!(
        ModuleAccess {
          module: identifier.get_value(self.source),
          item: item.get_value(self.source)
        },
        (identifier, item)
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

    let is_assignment_operator = self.current.ttype.is_assignment_operator();

    if (true, true) == (can_assign, is_assignment_operator) {
      let operator = self.current_advance();
      let right = self.expression()?;

      Ok(expression!(
        IndexAssignment {
          expression: Box::new(previous),
          index: Box::new(expression),
          assignment_operator: operators::Assignment::from_token(operator.ttype),
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
      if self.current.ttype == TokenType::RightParen {
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
        operator: operators::Binary::from(operator.ttype),
        left: Box::new(previous),
        right: Box::new(right),
      },
      (previous.span, right.span)
    ))
  }
}

// Types
impl<'source> Parser<'source> {
  fn optional_types(&mut self) -> Result<Option<TypeExpression<'source>>, Error> {
    if let TokenType::EndOfLine | TokenType::Comma = self.current.ttype {
      Ok(None)
    } else {
      Ok(Some(self.types()?))
    }
  }

  fn types(&mut self) -> TypeResult<'source> {
    let token = self.current;

    let mut t = match token.ttype {
      TokenType::Less => self.type_generic(),
      TokenType::Identifier | TokenType::Null | TokenType::True | TokenType::False => {
        self.next();

        if self.matches(TokenType::LeftParen) {
          self.type_param(token)
        } else {
          Ok(types!(Named(token.get_value(self.source)), token))
        }
      }
      TokenType::LeftParen => self.type_group(),
      _ => Err(Error::ExpectedType),
    };

    if self.matches(TokenType::LeftSquare) {
      t = self.type_list(t?);
    }

    match self.current.ttype {
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
    let opening_bracket = self.current_advance();

    if self.matches(TokenType::RightParen) {
      return self.type_function_body(opening_bracket, vec![]);
    };

    let types = self.types()?;
    match self.current.ttype {
      TokenType::Comma => {
        self.next();
        self.type_function(opening_bracket, types)
      }
      TokenType::RightParen => {
        let end_token = self.current_advance();

        if let TokenType::RightArrow | TokenType::FatRightArrow = self.current.ttype {
          self.type_function_body(opening_bracket, vec![types])
        } else {
          Ok(types!(Group(Box::new(types)), (opening_bracket, end_token)))
        }
      }
      TokenType::DotDot => {
        self.next();
        let ty = self.types()?;
        self.consume(TokenType::LeftParen, Error::ExpectedClosingBracket)?;
        self.type_function_body(opening_bracket, vec![ty])
      }
      _ => Err(Error::ExpectedClosingBrace)?,
    }
  }

  fn type_generic(&mut self) -> TypeResult<'source> {
    let opening_bracket = self.current_advance();

    if self.matches(TokenType::Greater) {
      Err(Error::ExpectedIdentifier)?;
    }

    let mut generics = Vec::new();
    loop {
      if self.matches(TokenType::Greater) {
        break;
      }

      let g = self.consume(TokenType::Identifier, Error::ExpectedIdentifier)?;
      generics.push(g.get_value(self.source));

      match self.current.ttype {
        TokenType::Comma => self.next(),
        TokenType::Greater => {
          self.next();
          break;
        }
        _ => Err(Error::ExpectedClosingAngle)?,
      };
    }

    let type_ = self.types()?;

    Ok(types!(
      WithGeneric(generics, Box::new(type_)),
      (opening_bracket, self.current)
    ))
  }

  fn type_param(&mut self, name: Token) -> TypeResult<'source> {
    let mut params = vec![self.types()?];

    while self.matches(TokenType::Comma) && self.current.ttype != TokenType::RightParen {
      params.push(self.types()?);
    }
    let end_token = self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;

    Ok(types!(
      Parameter(name.get_value(self.source), params),
      (name, end_token)
    ))
  }

  fn type_function(
    &mut self,
    start_token: Token,
    first_parameter: TypeExpression<'source>,
  ) -> TypeResult<'source> {
    let mut parameters = vec![first_parameter];
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

    self.type_function_body(start_token, parameters)
  }

  fn type_function_body(
    &mut self,
    start_token: Token,
    parameters: Vec<TypeExpression<'source>>,
  ) -> TypeResult<'source> {
    self.consume(TokenType::RightArrow, Error::ExpectedFunctionArrow)?;
    let return_type = self.types()?;

    Ok(types!(
      Function(Box::new(return_type), parameters),
      (start_token, self.current)
    ))
  }
}

impl<'source> Iterator for Parser<'source> {
  type Item = Result<Statement<'source>, Diagnostic>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.at_end() {
      return None;
    }

    match self.statement() {
      Ok(stmt) => Some(Ok(stmt)),
      Err(Error::EmptyStatement) => None,
      Err(err) => {
        let last_token = if self.current.ttype == TokenType::EndOfFile {
          self.previous
        } else {
          self.current
        };
        Some(Err(err.get_diagnostic(
          str::from_utf8(self.source).unwrap(),
          last_token,
        )))
      }
    }
  }
}

pub fn parse(source: &str) -> Result<Vec<Statement>, Diagnostic> {
  Parser::new(source).collect()
}

pub fn parse_type(source: &str) -> Result<TypeExpression, Diagnostic> {
  let mut parser = Parser::new(source);

  parser.types().map_err(|err| {
    let last_token = if parser.current.ttype == TokenType::EndOfFile {
      parser.previous
    } else {
      parser.current
    };

    err.get_diagnostic(str::from_utf8(source.as_bytes()).unwrap(), last_token)
  })
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
      assert_literal(&expression.expr, "hello world", LiteralType::String);
    } else {
      panic!("Expected group expression statement");
    }

    assert!(super::parse("(a)").is_ok());
    assert!(super::parse("(a").is_err());
    assert!(super::parse("(").is_err());
    assert!(super::parse("()").is_err());
  }

  #[test]
  fn should_parse_unary() {
    let statements = super::parse("!false\n").unwrap();

    if let Expr::Unary {
      operator,
      expression,
    } = unwrap_expression(&statements[0])
    {
      assert_literal(&expression.expr, "false", LiteralType::False);
      assert_eq!(*operator, operators::Unary::Not);
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
      assert_literal(&left.expr, "10", LiteralType::Number);
      assert_literal(&right.expr, "5", LiteralType::Number);
      assert_eq!(*operator, operators::Binary::Plus);
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
      assert_literal(&arguments[0].expr, "7", LiteralType::Number);
      assert_literal(&arguments[1].expr, "null", LiteralType::Null);
      assert_variable(&expression.expr, "function");
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
        assert_literal(&expression.expr, "null", LiteralType::Null);
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
        DeclarationIdentifier::Ordered(identifier) => assert_eq!(identifier.len(), 2),
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
      assert_variable(&expression.expr, "value");
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
  fn should_parse_imports() {
    let statements = super::parse("from x import {}\n").unwrap();
    if let Stmt::Import { module, items } = &statements[0].stmt {
      assert_eq!(items.len(), 0);
      assert_eq!(module, &"x");
    } else {
      panic!("Not import statement")
    }

    let statements = super::parse("from './abc/ef.bang' import { g, h, i }\n").unwrap();
    if let Stmt::Import { module, items } = &statements[0].stmt {
      assert_eq!(items.len(), 3);
      assert_eq!(module, &"./abc/ef.bang");
    } else {
      panic!("Not import statement")
    }

    assert!(super::parse("from 'agd import {}\n").is_err())
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
      assert_literal(&condition.expr, "true", LiteralType::True);
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
      assert_literal(&condition.expr, "true", LiteralType::True);
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
      assert_literal(&items[0].expr, "44", LiteralType::Number);
      assert_literal(&items[1].expr, "null", LiteralType::Null);
      assert_literal(&items[2].expr, "hello", LiteralType::String);
    } else {
      panic!("Expected list");
    }
  }

  #[test]
  fn should_parse_index() {
    let statements = super::parse("a[5]\n").unwrap();

    if let Expr::Index { expression, index } = unwrap_expression(&statements[0]) {
      assert_literal(&index.expr, "5", LiteralType::Number);
      assert_variable(&expression.expr, "a");
    } else {
      panic!("Expected list");
    }

    assert!(super::parse("a[5").is_err());
  }

  #[test]
  fn should_parse_format_string() {
    assert!(super::parse("'hello ${7}'").is_ok());
    assert!(super::parse("'hello ${   7 }'").is_ok());
    assert!(super::parse("'hello ${7} world'").is_ok());
    assert!(super::parse("'${7} world'").is_ok());
    assert!(super::parse("'${`hi`} world'").is_ok());
    assert!(super::parse("'${`hi`} \n world'").is_ok());
    assert!(super::parse("call('${7} world')").is_ok());
    assert!(super::parse("'hello ${ 7 } world ${false}!'").is_ok());
    assert!(super::parse("'Hello ${'I can interpolate'}, ${`multiple things`}'").is_ok());

    assert!(super::parse("'hello ${}'").is_err());
    assert!(super::parse("'hello ${7}").is_err());
    assert!(super::parse("`hello ${7}'").is_err());
    assert!(super::parse("'hello ${").is_err());
    assert!(super::parse("'hello ${'").is_err());
    assert!(super::parse("'hello ${77").is_err());
    assert!(super::parse("'Hello ${'I can interpolate'}, ${`multiple things`}").is_err());

    let with_import_after = "
let a = 'Hello ${'I can interpolate'}, ${`multiple things`}'
from maths import { sin }";
    assert_eq!(super::parse(with_import_after).unwrap().len(), 2);
  }

  #[test]
  fn should_parse_import() {
    assert!(super::parse("from maths import { sin }").is_ok());
    assert!(super::parse("from maths import { sin, cos }").is_ok());

    assert!(super::parse("from maths import sin }").is_err());
    assert!(super::parse("from maths import { sin ").is_err());
    assert!(super::parse("from maths import { sin }sin() ").is_err());
  }

  #[test]
  fn should_parse_type_groups() {
    assert!(super::parse("let a: (null | number)").is_ok());
    assert!(super::parse("let a: number").is_ok());

    assert!(super::parse("let a: (null").is_err());
  }

  #[test]
  fn should_parse_type_generics() {
    assert!(super::parse("let a: <T>(T) -> T").is_ok());
    assert!(super::parse("let a: <T>number").is_ok());
    assert!(super::parse("let a: <T, S, G>() -> null").is_ok());
    assert!(super::parse("let a: <T, S, G,>() -> null").is_ok());

    assert!(super::parse("let a: <>() -> null").is_err());
    assert!(super::parse("let a: <null>() -> null").is_err());
  }

  #[test]
  fn should_parse_type_param() {
    assert!(super::parse("let a: list(number)").is_ok());
    assert!(super::parse("let a: dict(string, number)").is_ok());
    assert!(super::parse("let a: dict(string, number,)").is_ok());
    assert!(super::parse("let a: list ( number ) ").is_ok());
    assert!(super::parse("let a: set(number?)").is_ok());
    assert!(super::parse("let a: magic(type | union)").is_ok());

    assert!(super::parse("let a: list number) = 3").is_err());
    assert!(super::parse("let a: list ()").is_err());
    assert!(super::parse("let a: list (number").is_err());
    assert!(super::parse("let a: list(number,,)").is_err());
    assert!(super::parse("let a: list(number,").is_err());
  }

  #[test]
  fn should_parse_module_access() {
    assert!(super::parse("maths::PI").is_ok());
    assert!(super::parse("maths::sin(7)").is_ok());

    assert!(super::parse("(maths)::sin").is_err());
    assert!(super::parse("::sin").is_err());
    assert!(super::parse("maths::").is_err());
    assert!(super::parse("(x + 4)::max").is_err());
  }

  #[test]
  fn should_parse_dictionary() {
    assert!(super::parse("{}").is_ok());
    assert!(super::parse("{ hello }").is_ok());
    assert!(super::parse("{ a, b, c, }").is_ok());
    assert!(super::parse("{ false: 5 }").is_ok());
    assert!(super::parse("{ false: 5, }").is_ok());
    assert!(super::parse("{ false: 5 + 4 }").is_ok());
    assert!(super::parse("{ hello: 5 + 4 }").is_ok());
    assert!(super::parse("{ 'false': 5 + 4 }").is_ok());
    assert!(super::parse("{ hello: }").is_err());
    assert!(super::parse("{ false }").is_err());
    assert!(super::parse("{ 3 }").is_err());
  }
}
