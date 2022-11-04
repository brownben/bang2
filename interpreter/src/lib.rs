#![feature(let_chains)]
#![feature(strict_provenance)]

pub mod chunk;
mod compiler;
pub mod context;
pub mod value;
mod vm;

pub use chunk::Chunk;
pub use compiler::compile;
pub use value::Value;
pub use vm::{RuntimeError, VM};

pub mod collections {
  pub use ahash::AHashMap as HashMap;
  pub use ahash::AHashSet as HashSet;
}
