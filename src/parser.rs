use crate::{
  ast::{Expr, Stmt},
  diagnostic::Diagnostic,
  tokens::{Token, TokenType},
};

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
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
  Comment,
}

impl Precedence {
  fn next(self) -> Self {
    match self {
      Self::None => Self::Assignment,
      Self::Assignment => Self::Or,
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
      TokenType::LeftParen => Self::Call,
      TokenType::Plus | TokenType::Minus => Self::Term,
      TokenType::Star | TokenType::Slash => Self::Factor,
      TokenType::BangEqual | TokenType::EqualEqual => Self::Equality,
      TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual => {
        Self::Comparison
      }
      TokenType::Comment => Self::Comment,
      _ => Self::None,
    }
  }
}

enum Error {
  ExpectedOpeningBracket,
  ExpectedClosingBracket,
  ExpectedExpression,
  ExpectedFunctionArrow,
  ExpectedNewLine,
  ExpectedIdentifier,
  InvalidAssignmentTarget,
  UnexpectedCharacter,
  UnterminatedString,
  EmptyStatement,
}
impl Error {
  fn get_title(&self) -> &'static str {
    match self {
      Self::ExpectedOpeningBracket => "Expected '('",
      Self::ExpectedClosingBracket => "Expected ')'",
      Self::ExpectedExpression => "Expected Expression",
      Self::ExpectedFunctionArrow => "Expected Funtion Arrow (-> / =>)",
      Self::ExpectedNewLine => "Expected New Line",
      Self::ExpectedIdentifier => "Expected Identifier",
      Self::InvalidAssignmentTarget => "Invalid Assignment Target",
      Self::UnexpectedCharacter => "Unexpected Character",
      Self::UnterminatedString => "Unterminated String",
      Self::EmptyStatement => unreachable!("EmptyStatement caught to return nothing"),
    }
  }

  fn get_message(&self, token: &Token) -> String {
    match self {
      Self::ExpectedOpeningBracket
      | Self::ExpectedClosingBracket
      | Self::ExpectedExpression
      | Self::ExpectedFunctionArrow
      | Self::ExpectedNewLine
      | Self::ExpectedIdentifier => format!("but recieved '{}'", token.value),
      Self::UnexpectedCharacter => format!("Unknown character '{}'", token.value),
      Self::UnterminatedString => {
        format!("Missing closing quote {}", &token.value[0..1])
      }
      Self::InvalidAssignmentTarget => "Can't assign to an expression, only a variable".to_string(),
      Self::EmptyStatement => unreachable!("EmptyStatement caught to return nothing"),
    }
  }

  fn get_diagnostic(&self, token: &Token) -> Diagnostic {
    Diagnostic {
      title: self.get_title().to_string(),
      message: self.get_message(token),
      lines: vec![token.line],
    }
  }
}

type ExpressionResult<'source> = Result<Expr<'source>, Error>;
type StatementResult<'source> = Result<Stmt<'source>, Error>;

struct Parser<'source> {
  tokens: &'source [Token<'source>],
  position: usize,
}

impl<'source> Parser<'source> {
  fn new(tokens: &'source [Token<'source>]) -> Self {
    Self {
      tokens,
      position: 0,
    }
  }

  fn at_end(&self) -> bool {
    self.position >= self.tokens.len()
  }

  fn next(&mut self) -> &'source Token<'source> {
    self.position += 1;
    let token = self.current();

    if token.ttype == TokenType::Whitespace {
      self.next()
    } else {
      token
    }
  }

  fn back(&mut self) -> &'source Token<'source> {
    self.position -= 1;
    let token = self.current();

    if token.ttype == TokenType::Whitespace {
      self.back()
    } else {
      token
    }
  }

  fn get(&self, position: usize) -> &'source Token<'source> {
    self.tokens.get(position).unwrap_or(&Token {
      ttype: TokenType::EndOfFile,
      value: "",
      line: 0,
      column: 0,
      start: 0,
      end: 0,
    })
  }

  fn current(&self) -> &'source Token<'source> {
    self.get(self.position)
  }

  fn current_advance(&mut self) -> &'source Token<'source> {
    let token = self.current();
    self.next();
    token
  }

  fn expect(
    &mut self,
    token_type: TokenType,
    message: Error,
  ) -> Result<&'source Token<'source>, Error> {
    let current = self.current();
    if current.ttype == token_type {
      Ok(current)
    } else {
      Err(message)
    }
  }

  fn consume(
    &mut self,
    token_type: TokenType,
    message: Error,
  ) -> Result<&'source Token<'source>, Error> {
    let result = self.expect(token_type, message)?;
    self.next();
    Ok(result)
  }

  fn consume_next(
    &mut self,
    token_type: TokenType,
    message: Error,
  ) -> Result<&'source Token<'source>, Error> {
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

  fn matches(&mut self, token_type: TokenType) -> bool {
    let matches = self.current().ttype == token_type;
    if matches {
      self.next();
    }
    matches
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
    self.matches(TokenType::EndOfLine);
    let token = self.current();

    let can_assign = precedence <= Precedence::Assignment;
    let prefix = self.prefix_rule(token.ttype, can_assign)?;
    let mut previous = vec![prefix];

    while precedence <= Precedence::from(self.next().ttype) {
      let token = self.current();

      if let Some(value) = self.infix_rule(token.ttype, previous.pop().unwrap())? {
        previous.push(value);
      }
    }
    self.back();

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
      TokenType::Unknown => Err(Error::UnexpectedCharacter),
      _ => Err(Error::ExpectedExpression),
    }
  }

  fn infix_rule(
    &mut self,
    token_type: TokenType,
    previous: Expr<'source>,
  ) -> Result<Option<Expr<'source>>, Error> {
    match token_type {
      TokenType::LeftParen => Ok(Some(self.call(previous)?)),
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
      | TokenType::QuestionQuestion => Ok(Some(self.binary(previous)?)),
      _ => Ok(None),
    }
  }
}

// Statements
impl<'source> Parser<'source> {
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
    while self.current().ttype == TokenType::EndOfLine {
      self.next();
    }

    let last = if self.position >= 1 {
      self.position - 1
    } else {
      0
    };
    let mut last_token = self.get(last);

    let depth = Parser::block_depth(last_token.value);

    if last_token.ttype != TokenType::Whitespace || depth == 0 {
      return self.stmt();
    }

    let mut statements = Vec::new();

    while last_token.ttype == TokenType::Whitespace
      && Parser::block_depth(last_token.value) >= depth
      && self.current().ttype != TokenType::EndOfFile
    {
      if Parser::block_depth(last_token.value) > depth {
        statements.push(self.statement()?);
        last_token = self.get(self.position - 1);
      } else {
        statements.push(self.stmt()?);
        last_token = self.get(self.position - 1);
      }
    }

    Ok(Stmt::Block { body: statements })
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
      _ => self.expression_statement(),
    }
  }

  fn var_declaration(&mut self) -> StatementResult<'source> {
    let token = self.current();
    let identifier = self.consume_next(TokenType::Identifier, Error::ExpectedIdentifier)?;
    let mut expression = if self.matches(TokenType::Equal) {
      Some(self.expression()?)
    } else {
      self.back();
      None
    };
    self.expect_newline()?;

    if let Some(Expr::Function {
      token,
      parameters,
      body,
      ..
    }) = expression
    {
      expression = Some(Expr::Function {
        token,
        parameters,
        body,
        name: Some(identifier.value),
      })
    }

    Ok(Stmt::Declaration {
      token,
      identifier,
      expression,
    })
  }

  fn return_statement(&mut self) -> StatementResult<'source> {
    let token = self.current_advance();
    let expression = if self.matches(TokenType::EndOfLine) {
      None
    } else {
      let exp = Some(self.expression()?);
      self.expect_newline()?;
      exp
    };

    Ok(Stmt::Return { token, expression })
  }

  fn if_statement(&mut self) -> StatementResult<'source> {
    let if_token = self.current();
    self.consume_next(TokenType::LeftParen, Error::ExpectedOpeningBracket)?;
    let condition = self.expression()?;
    self.consume_next(TokenType::RightParen, Error::ExpectedClosingBracket)?;

    self.matches(TokenType::EndOfLine);
    let body = self.statement()?;

    while self.matches(TokenType::EndOfLine) {}

    let (else_token, otherwise) = if self.current().ttype == TokenType::Else {
      let else_token = self.current_advance();
      while self.matches(TokenType::EndOfLine) {}

      (Some(else_token), Some(Box::new(self.statement()?)))
    } else {
      (None, None)
    };

    Ok(Stmt::If {
      if_token,
      else_token,
      condition,
      then: Box::new(body),
      otherwise,
    })
  }

  fn while_statement(&mut self) -> StatementResult<'source> {
    let token = self.current();
    self.consume_next(TokenType::LeftParen, Error::ExpectedOpeningBracket)?;
    let condition = self.expression()?;
    self.consume_next(TokenType::RightParen, Error::ExpectedClosingBracket)?;
    self.matches(TokenType::EndOfLine);
    let body = Box::new(self.statement()?);

    Ok(Stmt::While {
      token,
      condition,
      body,
    })
  }

  fn comment_statement(&mut self) -> StatementResult<'source> {
    let token = self.current_advance();
    self.expect_newline()?;

    Ok(Stmt::Comment { token })
  }

  fn expression_statement(&mut self) -> StatementResult<'source> {
    let expression = self.expression()?;
    self.expect_newline()?;

    Ok(Stmt::Expression { expression })
  }
}

// Expressions
impl<'source> Parser<'source> {
  fn expression(&mut self) -> ExpressionResult<'source> {
    self.parse_expression(Precedence::Assignment)
  }

  fn function(&mut self) -> ExpressionResult<'source> {
    let token = self.current_advance();

    let mut parameters = Vec::new();
    loop {
      self.matches(TokenType::EndOfLine);
      if self.matches(TokenType::RightParen) {
        break;
      }

      let parameter = self.consume(TokenType::Identifier, Error::ExpectedIdentifier)?;
      parameters.push(parameter);

      if !self.matches(TokenType::Comma) {
        self.consume(TokenType::RightParen, Error::ExpectedClosingBracket)?;
        break;
      }
    }

    let body = if self.matches(TokenType::FatRightArrow) {
      Ok(Stmt::Return {
        token: self.current(),
        expression: Some(self.expression()?),
      })
    } else if self.matches(TokenType::RightArrow) {
      self.matches(TokenType::EndOfLine);
      let statement = self.statement()?;
      self.back();
      Ok(statement)
    } else {
      Err(Error::ExpectedFunctionArrow)
    }?;

    Ok(Expr::Function {
      token,
      body: Box::new(body),
      parameters,
      name: None,
    })
  }

  fn grouping(&mut self) -> ExpressionResult<'source> {
    if self.is_function_bracket() {
      return self.function();
    }

    let token = self.current_advance();
    let expression = self.expression()?;
    self.next();
    let end_token = self.expect(TokenType::RightParen, Error::ExpectedClosingBracket)?;

    Ok(Expr::Group {
      token,
      end_token,
      expression: Box::new(expression),
    })
  }

  fn unary(&mut self) -> ExpressionResult<'source> {
    let token = self.current_advance();
    let expression = self.parse_expression(Precedence::Unary)?;

    Ok(Expr::Unary {
      operator: token,
      expression: Box::new(expression),
    })
  }

  fn literal(&mut self) -> ExpressionResult<'source> {
    let token = self.current();
    let string = token.value;

    let value = if token.ttype != TokenType::String {
      Ok(string)
    } else if string[0..1] == string[string.len() - 1..string.len()] {
      Ok(&string[1..string.len() - 1])
    } else {
      Err(Error::UnterminatedString)
    }?;

    Ok(Expr::Literal { token, value })
  }

  fn variable(&mut self, can_assign: bool) -> ExpressionResult<'source> {
    let identifier = self.current();
    let is_additional_operator = matches!(
      self.next().ttype,
      TokenType::PlusEqual | TokenType::MinusEqual | TokenType::StarEqual | TokenType::SlashEqual
    );

    if (true, true) == (can_assign, is_additional_operator) {
      let operator = self.current_advance();

      Ok(Expr::Assignment {
        identifier,
        expression: Box::new(Expr::Binary {
          operator,
          left: Box::new(Expr::Variable { token: identifier }),
          right: Box::new(self.expression()?),
        }),
      })
    } else if self.matches(TokenType::Equal) && can_assign {
      Ok(Expr::Assignment {
        identifier,
        expression: Box::new(self.expression()?),
      })
    } else {
      self.back();
      Ok(Expr::Variable { token: identifier })
    }
  }

  fn call(&mut self, previous: Expr<'source>) -> ExpressionResult<'source> {
    let token = self.current_advance();

    let mut arguments = Vec::new();
    let end_token = loop {
      self.matches(TokenType::EndOfLine);
      if self.current().ttype == TokenType::RightParen {
        break self.current();
      }

      arguments.push(self.expression()?);
      self.next();

      if !self.matches(TokenType::Comma) {
        break self.expect(TokenType::RightParen, Error::ExpectedClosingBracket)?;
      }
    };

    Ok(Expr::Call {
      expression: Box::new(previous),
      token,
      arguments,
      end_token,
    })
  }

  fn comment(&mut self, previous: Expr<'source>) -> Expr<'source> {
    let token = self.current();

    Expr::Comment {
      expression: Box::new(previous),
      token,
    }
  }

  fn binary(&mut self, previous: Expr<'source>) -> ExpressionResult<'source> {
    let operator = self.current_advance();
    let precedence = Precedence::from(operator.ttype);
    let right = self.parse_expression(precedence.next())?;

    Ok(Expr::Binary {
      operator,
      left: Box::new(previous),
      right: Box::new(right),
    })
  }
}

pub fn parse<'s>(tokens: &'s [Token<'s>]) -> Result<Vec<Stmt<'s>>, Diagnostic> {
  let mut parser = Parser::new(tokens);
  let mut statements = Vec::new();

  while !parser.at_end() {
    match parser.statement() {
      Ok(stmt) => statements.push(stmt),
      Err(Error::EmptyStatement) => {}
      Err(err) => {
        return Err(err.get_diagnostic(parser.current()));
      }
    }
  }

  Ok(statements)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tokens::tokenize;

  fn assert_literal(expr: &Expr<'_>, value: &str, literal_type: TokenType) {
    match expr {
      Expr::Literal { token, .. } => {
        assert_eq!(token.value, value);
        assert_eq!(token.ttype, literal_type);
      }
      _ => panic!("Expected literal"),
    }
  }

  fn assert_variable(expr: &Expr<'_>, name: &str) {
    match expr {
      Expr::Variable { token } => {
        assert_eq!(token.value, name);
        assert_eq!(token.ttype, TokenType::Identifier);
      }
      _ => panic!("Expected literal"),
    }
  }

  #[test]
  fn should_error_on_unknown_character() {
    let tokens = tokenize("&");
    let result = super::parse(&tokens);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message, "Unknown character '&'");
  }

  #[test]
  fn should_parse_group() {
    let tokens = tokenize("('hello world')\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::Expression {
      expression: Expr::Group { expression, .. },
    } = &statements[0]
    {
      assert_literal(&**expression, "'hello world'", TokenType::String);
    } else {
      panic!("Expected group expression statement");
    }
  }

  #[test]
  fn should_parse_unary() {
    let tokens = tokenize("!false\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::Expression {
      expression: Expr::Unary {
        operator,
        expression,
      },
    } = &statements[0]
    {
      assert_literal(&**expression, "false", TokenType::False);
      assert_eq!(operator.ttype, TokenType::Bang);
    } else {
      panic!("Expected unary expression statement");
    }
  }

  #[test]
  fn should_parse_binary() {
    let tokens = tokenize("10 + 5\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::Expression {
      expression: Expr::Binary {
        operator,
        left,
        right,
      },
    } = &statements[0]
    {
      assert_literal(&**left, "10", TokenType::Number);
      assert_literal(&**right, "5", TokenType::Number);
      assert_eq!(operator.ttype, TokenType::Plus);
    } else {
      panic!("Expected binary expression statement");
    }
  }

  #[test]
  fn should_parse_call() {
    let tokens = tokenize("function(7, null)\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::Expression {
      expression: Expr::Call {
        expression,
        arguments,
        ..
      },
    } = &statements[0]
    {
      assert_literal(&arguments[0], "7", TokenType::Number);
      assert_literal(&arguments[1], "null", TokenType::Null);
      assert_variable(expression, "function");
    } else {
      panic!("Expected binary expression statement");
    }
  }

  #[test]
  fn should_parse_function() {
    let tokens = tokenize("() => null\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::Expression {
      expression:
        Expr::Function {
          parameters,
          body:
            box Stmt::Return {
              expression: Some(expression),
              ..
            },
          ..
        },
      ..
    } = &statements[0]
    {
      assert_eq!(parameters.len(), 0);
      assert_literal(expression, "null", TokenType::Null);
    } else {
      panic!("Expected return statement");
    }
  }

  #[test]
  fn should_parse_variable_declaration_with_initalizer() {
    let tokens = tokenize("let a = null\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::Declaration {
      identifier,
      expression: Some(Expr::Literal { token, .. }),
      ..
    } = &statements[0]
    {
      assert_eq!(identifier.value, "a");
      assert_eq!(token.value, "null");
      assert_eq!(token.ttype, TokenType::Null);
    } else {
      panic!("Expected declaration statement");
    }
  }

  #[test]
  fn should_parse_variable_declaration_without_initalizer() {
    let tokens = tokenize("let b\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::Declaration {
      identifier,
      expression: None,
      ..
    } = &statements[0]
    {
      assert_eq!(identifier.value, "b");
    } else {
      panic!("Expected declaration statement");
    }
  }

  #[test]
  fn should_parse_return_with_value() {
    let tokens = tokenize("return value\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::Return {
      expression: Some(expression),
      ..
    } = &statements[0]
    {
      assert_variable(expression, "value");
    } else {
      panic!("Expected return statement");
    }
  }

  #[test]
  fn should_parse_return_without_value() {
    let tokens = tokenize("return\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::Return {
      expression: None,
      token,
    } = &statements[0]
    {
      assert_eq!(token.value, "return");
    } else {
      panic!("Expected return statement");
    }
  }

  #[test]
  fn should_parse_while() {
    let tokens = tokenize("while(7) doStuff\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::While {
      condition: Expr::Literal {
        token: condition, ..
      },
      body: box Stmt::Expression { expression },
      ..
    } = &statements[0]
    {
      assert_eq!(condition.value, "7");
      assert_variable(expression, "doStuff");
    } else {
      panic!("Expected while statement");
    }
  }

  #[test]
  fn should_parse_if_else() {
    let tokens = tokenize("if (true) doStuff\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::If {
      condition,
      then: box Stmt::Expression { expression },
      otherwise,
      ..
    } = &statements[0]
    {
      assert_literal(condition, "true", TokenType::True);
      assert_variable(expression, "doStuff");
      assert!(otherwise.is_none());
    } else {
      panic!("Expected if statement");
    }
  }

  #[test]
  fn should_parse_if_without_else() {
    let tokens = tokenize("if (true)\n\tdoStuff\nelse\n\tdoOtherStuff\n");
    let statements = super::parse(&tokens).unwrap();

    if let Stmt::If {
      condition,
      then: box Stmt::Block { body: then_body },
      otherwise: Some(box Stmt::Block { body: else_body }),
      ..
    } = &statements[0]
    {
      assert_literal(condition, "true", TokenType::True);
      assert_eq!(then_body.len(), 1);
      assert_eq!(else_body.len(), 1);
    } else {
      panic!("Expected if statement");
    }
  }

  #[test]
  fn should_parse_block() {
    let tokens = tokenize("a\n\tdoStuff\n\totherStuff\n\tmoreStuff\n");
    let statements = super::parse(&tokens).unwrap();

    assert_eq!(statements.len(), 2);

    if let Stmt::Block { body } = &statements[1] {
      assert_eq!(body.len(), 3);
    } else {
      panic!("Expected block statement");
    }
  }
}
