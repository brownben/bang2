#![feature(let_chains)]
#![feature(strict_provenance)]

pub mod chunk;
mod compiler;
pub mod context;
pub mod value;
mod vm;

pub use chunk::Chunk;
pub use compiler::{compile, compile_into_chunk};
pub use value::Value;
pub use vm::{RuntimeError, VM};

pub mod errors {
  pub use super::vm::RuntimeError as Runtime;
  pub use super::vm::{StackTraceLocation, StackTraceLocationKind};
}

pub mod collections {
  pub use rustc_hash::FxHashMap as HashMap;
  pub use rustc_hash::FxHashSet as HashSet;
}
