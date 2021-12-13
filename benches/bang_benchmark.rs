pub use bang;
pub use bang::{Chunk, CompileError, Value};

pub fn compile(source: &str) -> Result<Chunk, CompileError> {
  let ast = bang::parse(source)?;
  bang::compile(ast)
}

#[macro_export]
macro_rules! bang_benchmark {
  ($name:ident, $source:expr) => {
    mod $name {
      extern crate test;
      use super::*;
      use test::Bencher;

      #[bench]
      fn to_bytecode(b: &mut Bencher) {
        b.iter(|| compile($source));
      }

      #[bench]
      fn vm(b: &mut Bencher) {
        match compile($source) {
          Ok(chunk) => {
            b.iter(|| bang::run(chunk.clone()));
          }
          _ => {}
        }
      }

      #[bench]
      fn all(b: &mut Bencher) {
        b.iter(|| match compile($source) {
          Ok(chunk) => {
            bang::run(chunk);
          }
          _ => {}
        })
      }
    }
  };
}
