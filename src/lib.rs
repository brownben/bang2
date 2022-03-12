#![feature(box_patterns)]
use ahash::AHashMap as HashMap;
use std::rc::Rc;

mod ast;
mod builtins;
mod chunk;
mod compiler;
mod diagnostic;
mod linter;
mod parser;
pub mod print;
mod tokens;
mod value;
mod vm;

// A error or warning from the language
pub use diagnostic::Diagnostic;

// Tokenise a source string
pub use tokens::{tokenize, Token};

// Parse a slice of tokens into an AST
pub use ast::{Expr, Stmt};
pub use parser::parse;

// Check an AST for common problems
pub use linter::lint;

// Compile an AST into a chunk of bytecode
pub use chunk::Chunk;
pub use compiler::compile;

// Run a chunk of bytecode
pub use vm::{run, VM};

// A value from the virtual machine
pub use value::Value;

// Interpret a string of code
pub fn interpret(source: &str) -> Result<HashMap<Rc<str>, Value>, Diagnostic> {
  let tokens = tokens::tokenize(source);
  let ast = parser::parse(&tokens)?;
  let chunk = compiler::compile(&ast)?;

  vm::run(chunk)
}
