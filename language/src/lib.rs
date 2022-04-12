#![feature(box_patterns, let_chains)]
use ahash::AHashMap as HashMap;
use std::rc::Rc;

pub mod ast;
mod builtins;
mod chunk;
mod compiler;
mod diagnostic;
mod parser;
pub mod print;
mod tokens;
mod value;
mod vm;

// A error or warning from the language
pub use diagnostic::Diagnostic;

// Tokenise a source string
pub use tokens::{tokenize, LineNumber, Token};

// Parse a slice of tokens into an AST
pub use parser::{parse, parse_number};

// Compile an AST into a chunk of bytecode
pub use chunk::Chunk;
pub use compiler::compile;

// Run a chunk of bytecode
pub use vm::{run, VM};

// A value from the virtual machine
pub use value::Value;

// Interpret a string of code
pub type VMGlobals = HashMap<Rc<str>, Value>;
pub fn interpret(source: &str) -> Result<VMGlobals, Diagnostic> {
  let tokens = tokens::tokenize(source);
  let ast = parser::parse(source, &tokens)?;
  let chunk = compiler::compile(source, &ast)?;

  vm::run(chunk)
}
