mod ast;
mod chunk;
mod diagnostics;

fn remove_carriage_returns(value: &str) -> String {
  str::replace(value, "\r", "")
}

pub use ast::print as ast;
pub use chunk::print as chunk;
pub use diagnostics::runtime_error;
pub use diagnostics::{error, error_message};
pub use diagnostics::{warning, warning_message};
