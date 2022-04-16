use crate::tokens::LineNumber;

#[derive(Debug, PartialEq)]
pub struct Diagnostic {
  pub title: String,
  pub message: String,
  pub lines: Vec<LineNumber>,
}
