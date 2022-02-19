use crate::tokens::LineNumber;

#[derive(Debug)]
pub struct Diagnostic {
  pub title: String,
  pub message: String,
  pub lines: Vec<LineNumber>,
}
