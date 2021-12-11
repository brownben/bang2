use crate::token::{LineNumber, Token};

#[derive(Debug, Clone, Copy)]
pub enum Error {
  UnterminatedString,
  UnknownCharacter,
  VariableAlreadyExists,
  MissingVariableName,
  MissingEndOfFile,
  InvalidAssignmentTarget,
  MissingBracketBeforeCondition,
  MissingBracketAfterCondition,
  ExpectedNewLine,
  ExpectedEndOfBlock,
  ExpectedBracket,
  ExpectedExpression,
  TooManyConstants,
  TooBigJump,
  UnknownBinaryOperator,
  UnknownUnaryOperator,
}

#[derive(Debug)]
pub struct CompileError {
  pub error: Error,
  pub token: Token,
}

pub struct RuntimeError {
  pub line_number: LineNumber,
  pub message: String,
}

pub struct Diagnostic {
  pub message: String,
  pub label: String,
  pub note: String,
}

pub fn get_message(source: &[char], error: &Error, token: &Token) -> Diagnostic {
  match error {
    Error::UnterminatedString => Diagnostic {
      message: "Unterminated String".to_string(),
      label: format!("Missing closing quote {}", token.get_value(source)),
      note: format!("Add {} to close the string", token.get_value(source)),
    },
    Error::UnknownCharacter => Diagnostic {
      message: "Unknown Character".to_string(),
      label: format!("Unknown character '{}'", token.get_value(source)),
      note: "Try deleting the character".to_string(),
    },
    Error::UnknownBinaryOperator => Diagnostic {
      message: "Unknown Binary Operator".to_string(),
      label: format!("Unknown binary operator '{}'", token.get_value(source)),
      note: "Are you using the correct operator?".to_string(),
    },
    Error::UnknownUnaryOperator => Diagnostic {
      message: "Unknown Unary Operator".to_string(),
      label: format!("Unknown unary operator '{}'", token.get_value(source)),
      note: "Are you using the correct operator?".to_string(),
    },

    Error::VariableAlreadyExists => Diagnostic {
      message: "Redefining Existing Variable".to_string(),
      label: format!("Variable '{}' already exists", token.get_value(source)),
      note: "You could try a new name for your variable".to_string(),
    },
    Error::MissingVariableName => Diagnostic {
      message: "Expected Variable Name".to_string(),
      label: "Variable not assigned a name".to_string(),
      note: "Add the name for your variable".to_string(),
    },
    Error::MissingEndOfFile => Diagnostic {
      message: "Missing End of File".to_string(),
      label: "No End of File token".to_string(),
      note: "This is likely a problem with the compiler rather than your code".to_string(),
    },
    Error::InvalidAssignmentTarget => Diagnostic {
      message: "Invalid Assignment Target".to_string(),
      label: "Assignment target is not a variable".to_string(),
      note: "Assign to a variable rather than an expression".to_string(),
    },
    Error::MissingBracketBeforeCondition => Diagnostic {
      message: "Expected Bracket Before Condition".to_string(),
      label: "Expected '(' before condition".to_string(),
      note: "Add a ( before the condition".to_string(),
    },
    Error::MissingBracketAfterCondition => Diagnostic {
      message: "Expected Bracket After Condition".to_string(),
      label: "Expected ')' after condition".to_string(),
      note: "Add a ) after the condition".to_string(),
    },
    Error::ExpectedNewLine => Diagnostic {
      message: "Expected New Line After Expression".to_string(),
      label: "Expected a new line here".to_string(),
      note: "Add a new line".to_string(),
    },
    Error::ExpectedEndOfBlock => Diagnostic {
      message: "Expected End of Block".to_string(),
      label: "Expected the block to end here".to_string(),
      note: "Try dedenting the next line".to_string(),
    },
    Error::TooManyConstants => Diagnostic {
      message: "Too Many Constants".to_string(),
      label: "Couldn't add constant, as already too many in chunk".to_string(),
      note: "This is likely to be an issue with the compiler".to_string(),
    },
    Error::TooBigJump => Diagnostic {
      message: "Jump Too Large".to_string(),
      label: "Couldn't construct bytecode, as block too large".to_string(),
      note: "This is likely to be an issue with the compiler".to_string(),
    },
    Error::ExpectedBracket => Diagnostic {
      message: "Expected Closing Bracket".to_string(),
      label: "Expected ')' after expression".to_string(),
      note: "Add a ) to close the expression".to_string(),
    },
    Error::ExpectedExpression => Diagnostic {
      message: "Expected Expression".to_string(),
      label: "Expected expression here".to_string(),
      note: "Add an expression here".to_string(),
    },
  }
}
