#![feature(let_chains)]
#![feature(strict_provenance)]

mod chunk;
mod compiler;
mod context;
mod value;
mod vm;

pub use chunk::{Chunk, OpCode};
pub use compiler::compile;
pub use context::{Context, Empty as EmptyContext, ImportValue};
pub use value::{calculate_index, NativeFunction, Object, Value};
pub use vm::{RuntimeError, VM};

pub use ahash::AHashSet as HashSet;
