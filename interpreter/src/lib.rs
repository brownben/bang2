#![feature(let_chains)]

mod builtins;
mod chunk;
mod compiler;
mod value;
mod vm;

// Compile an AST into a chunk of bytecode
pub use chunk::{Chunk, OpCode};
pub use compiler::compile;

// Run a chunk of bytecode
pub use vm::RuntimeError;
pub use vm::{run, VM};

// A value from the virtual machine
pub use value::Value;
