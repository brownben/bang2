pub use bang;
pub use bang::{Chunk, CompileError, Value};

pub fn compile(source: &str) -> Result<Chunk, CompileError> {
  let ast = bang::parse(source)?;
  bang::compile(ast)
}

pub fn run(source: &str) {
  match compile(source) {
    Ok(chunk) => {
      match bang::run(chunk) {
        Ok(_) => (),
        Err(_) => (),
      };
    }
    _ => {}
  }
}

#[macro_export]
macro_rules! bang_benchmark {
  ($name:ident, $source:expr) => {
    #[bench]
    fn $name(b: &mut Bencher) {
      b.iter(|| run($source))
    }
  };
}
