use crate::token::{LineNumber, Token};

#[derive(Debug, Clone, Copy)]
pub enum Error {
  UnterminatedString,
  UnknownCharacter,
  VariableAlreadyExists,
  MissingVariableName,
  MissingEndOfFile,
  InvalidAssignmentTarget,
  MissingBracketBeforeParameters,
  MissingBracketBeforeCondition,
  MissingBracketAfterCondition,
  ExpectedNewLine,
  ExpectedEndOfBlock,
  ExpectedBracket,
  ExpectedExpression,
  TooManyArguments,
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
  pub message: String,
  pub line_numbers: Vec<LineNumber>,
}

pub struct Diagnostic {
  pub message: String,
  pub note: Option<String>,
}

pub fn get_message(source: &[char], error: &Error, token: &Token) -> Diagnostic {
  match error {
    Error::UnterminatedString => Diagnostic {
      message: format!(
        "Unterminated String, Missing closing quote {}",
        token.get_value(source)
      ),
      note: None,
    },
    Error::UnknownCharacter => Diagnostic {
      message: format!("Unknown Character '{}'", token.get_value(source)),
      note: None,
    },
    Error::UnknownBinaryOperator => Diagnostic {
      message: format!("Unknown Binary Operator '{}'", token.get_value(source)),
      note: None,
    },
    Error::UnknownUnaryOperator => Diagnostic {
      message: format!("Unknown Unary Operator '{}'", token.get_value(source)),
      note: None,
    },

    Error::VariableAlreadyExists => Diagnostic {
      message: format!(
        "Redefining Exisiting Variable '{}'",
        token.get_value(source)
      ),
      note: None,
    },
    Error::MissingVariableName => Diagnostic {
      message: "Expected Variable Name".to_string(),
      note: None,
    },

    Error::InvalidAssignmentTarget => Diagnostic {
      message: "Invalid Assignment Target, Must be a Variable".to_string(),
      note: None,
    },
    Error::MissingBracketBeforeCondition => Diagnostic {
      message: "Expected Bracket '(' Before Condition".to_string(),
      note: None,
    },
    Error::MissingBracketAfterCondition => Diagnostic {
      message: "Expected Closing Bracket ')' After Condition".to_string(),
      note: None,
    },
    Error::MissingBracketBeforeParameters => Diagnostic {
      message: "Expected Bracket '(' Before Parameters".to_string(),
      note: None,
    },
    Error::ExpectedNewLine => Diagnostic {
      message: "Expected New Line After Expression".to_string(),
      note: None,
    },
    Error::ExpectedEndOfBlock => Diagnostic {
      message: "Expected End of Block".to_string(),
      note: None,
    },
    Error::MissingEndOfFile => Diagnostic {
      message: "Missing End of File".to_string(),
      note: Some("This is likely to be an issue with the compiler".to_string()),
    },
    Error::TooManyArguments => Diagnostic {
      message: "Too Many Arguments (Max 255)".to_string(),
      note: None,
    },
    Error::TooManyConstants => Diagnostic {
      message: "Too Many Constants".to_string(),
      note: Some("This is likely to be an issue with the compiler".to_string()),
    },
    Error::TooBigJump => Diagnostic {
      message: "Jump Too Large".to_string(),
      note: Some("This is likely to be an issue with the compiler".to_string()),
    },
    Error::ExpectedBracket => Diagnostic {
      message: "Expected Closing Bracket ')' After Expression".to_string(),
      note: None,
    },
    Error::ExpectedExpression => Diagnostic {
      message: "Expected Expression".to_string(),
      note: None,
    },
  }
}
