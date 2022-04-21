mod ast;
mod chunk;
mod diagnostics;
mod tokens;

fn remove_carriage_returns(value: &str) -> String {
  str::replace(value, "\r", "")
}

pub use ast::print as ast;
pub use chunk::print as chunk;
pub use diagnostics::error;
pub use diagnostics::warning;
pub use tokens::print as tokens;
