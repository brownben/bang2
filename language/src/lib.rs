#![feature(box_patterns, let_chains)]
use ahash::AHashMap as HashMap;
use std::rc::Rc;

mod ast;
mod builtins;
mod chunk;
mod compiler;
mod diagnostic;
mod formatter;
mod linter;
mod parser;
pub mod print;
mod tokens;
mod typechecker;
mod value;
mod vm;

// A error or warning from the language
pub use diagnostic::Diagnostic;

// Tokenise a source string
pub use tokens::{tokenize, Token};

// Parse a slice of tokens into an AST
pub use ast::{expression, statement, types};
pub use parser::parse;

// Check an AST for common problems
pub use linter::lint;

// Format an AST in a opinionated manner
pub use formatter::format;

// Typecheck the code
pub use typechecker::typecheck;

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
  let ast = parser::parse(source, &tokens)?;
  let chunk = compiler::compile(source, &ast)?;

  vm::run(chunk)
}
