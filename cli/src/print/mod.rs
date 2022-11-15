mod ast;
mod chunk;
mod diagnostics;

fn remove_carriage_returns(value: &str) -> String {
  str::replace(value, "\r", "")
}

pub use ast::print as ast;
pub use chunk::print as chunk;
pub use diagnostics::{code_frame, error_message, stack_trace, warning_message};
