mod ast;
mod diagnostics;

fn remove_carriage_returns(value: &str) -> String {
  str::replace(value, "\r", "")
}

pub use ast::print as ast;
pub use diagnostics::{code_frame, error_message, stack_trace, warning_message};
