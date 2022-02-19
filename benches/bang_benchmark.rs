#[macro_export]
macro_rules! bang_benchmark {
  ($name:ident, $source:expr) => {
    mod $name {
      extern crate test;
      use bang;
      use test::Bencher;

      fn compile(source: &str) -> Result<bang::Chunk, bang::Diagnostic> {
        let tokens = bang::tokenize(source);
        let ast = bang::parse(&tokens)?;
        bang::compile(&ast)
      }

      #[allow(unused_must_use)]
      #[bench]
      fn to_bytecode(b: &mut Bencher) {
        b.iter(|| compile($source));
      }

      #[allow(unused_must_use)]
      #[bench]
      fn vm(b: &mut Bencher) {
        let chunk = compile($source).unwrap();
        b.iter(|| bang::run(chunk.clone()));
      }

      #[allow(unused_must_use)]
      #[bench]
      fn all(b: &mut Bencher) {
        b.iter(|| bang::interpret($source))
      }
    }
  };
}
