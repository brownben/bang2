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
      use test::Bencher;

      mod bang {
        pub use bang_interpreter::*;
        pub use bang_std::*;
        pub use bang_syntax::*;
      }

      const SOURCE: &'static str = $source;

      fn compile() -> Result<bang::Chunk, bang::Diagnostic> {
        let ast = bang::parse(SOURCE)?;
        bang::compile(SOURCE, &ast, &bang::StdContext)
      }

      #[bench]
      fn parse(b: &mut Bencher) {
        b.iter(|| bang::parse(SOURCE));
      }

      #[bench]
      fn to_bytecode(b: &mut Bencher) {
        b.iter(|| compile());
      }

      #[bench]
      fn vm(b: &mut Bencher) {
        let chunk = compile().unwrap();

        b.iter(|| {
          let mut vm = bang::VM::new(&bang::StdContext);
          vm.run(&chunk).expect("No runtime errors");
        });
      }

      #[bench]
      fn all(b: &mut Bencher) {
        b.iter(|| {
          let chunk = compile().expect("Successful compile");
          let mut vm = bang::VM::new(&bang::StdContext);
          vm.run(&chunk).expect("No runtime errors");
        })
      }
    }
  };
}

bang_benchmark!(arithmetic, examples "arithmeticBench.bang");

bang_benchmark!(iterative_fibonacci, examples "iterativeFibonacci.bang");

bang_benchmark!(recursive_fibonacci, examples "recursiveFibonacci.bang");

bang_benchmark!(syntax, examples "syntax.bang");

bang_benchmark!(bubble_sort, examples "bubbleSort.bang");
