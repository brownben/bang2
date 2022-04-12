#![feature(test)]

#[cfg(not(windows))]
macro_rules! file_separator {
  () => {
    "/"
  };
}

#[cfg(windows)]
macro_rules! file_separator {
  () => {
    r#"\"#
  };
}

macro_rules! bang_benchmark {
  ($name:ident, examples $file: literal) => {
    bang_benchmark!(
      $name,
      include_str!(concat!(
        "..",
        file_separator!(),
        "..",
        file_separator!(),
        "examples",
        file_separator!(),
        $file
      ))
    );
  };

  ($name:ident, $source:expr) => {
    mod $name {
      extern crate test;
      use bang;
      use test::Bencher;

      const SOURCE: &'static str = $source;

      fn compile() -> Result<bang::Chunk, bang::Diagnostic> {
        let tokens = bang::tokenize(SOURCE);
        let ast = bang::parse(SOURCE, &tokens)?;
        bang::compile(SOURCE, &ast)
      }

      #[bench]
      fn to_bytecode(b: &mut Bencher) {
        b.iter(|| compile());
      }

      #[bench]
      fn vm(b: &mut Bencher) {
        let chunk = compile().unwrap();
        b.iter(|| bang::run(chunk.clone()));
      }

      #[bench]
      fn all(b: &mut Bencher) {
        b.iter(|| bang::interpret(SOURCE))
      }
    }
  };
}

bang_benchmark!(arithmetic, examples "arithmeticBench.bang");

bang_benchmark!(iterative_fibonacci, examples "iterativeFibonacci.bang");

bang_benchmark!(recursive_fibonacci, examples "recursiveFibonacci.bang");

bang_benchmark!(syntax, examples "syntax.bang");
