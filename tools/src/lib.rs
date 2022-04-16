#![feature(let_chains, box_patterns)]

mod builtins;
mod formatter;
mod linter;
pub(crate) mod typechecker;

// Check an AST for common problems
pub use linter::lint;

// Format an AST in a opinionated manner
pub use formatter::format;

// Typecheck the code
pub use typechecker::typecheck;
