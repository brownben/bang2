#![feature(let_chains)]

pub mod ast;
mod parser;
mod tokens;

pub use parser::Diagnostic;
pub use parser::{parse, Parser};
pub use tokens::LineNumber;

pub type Ast<'a> = Vec<ast::statement::Statement<'a>>;
