#![feature(test)]
extern crate test;
use test::{black_box, Bencher};

pub mod bang {
  pub use bang_interpreter::*;
  pub use bang_std::*;
  pub use bang_syntax::*;
}

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

macro_rules! read_example_file {
  ($file_name:literal) => {{
    include_str!(concat!(
      "..",
      file_separator!(),
      "..",
      file_separator!(),
      "examples",
      file_separator!(),
      $file_name
    ))
  }};
}

macro_rules! benchmark_from_file {
  (example, $name:ident, $file:literal) => {
    mod $name {
      use super::*;
      static FILE: &str = read_example_file!($file);

      #[bench]
      fn parse(b: &mut Bencher) {
        b.iter(|| bang::parse(black_box(FILE)));
      }

      #[bench]
      fn compile(b: &mut Bencher) {
        b.iter(|| bang::compile(black_box(FILE)));
      }

      #[bench]
      fn run(b: &mut Bencher) {
        let chunk = bang::compile(FILE).unwrap();
        b.iter(|| {
          let context = bang::StdContext::default();
          bang::VM::new(&context).run(black_box(&chunk)).unwrap();
        })
      }

      #[bench]
      fn all(b: &mut Bencher) {
        b.iter(|| {
          let chunk = bang::compile(FILE).unwrap();
          let context = bang::StdContext::default();
          bang::VM::new(&context).run(black_box(&chunk)).unwrap();
        })
      }
    }
  };
}

benchmark_from_file!(example, arithmetic_bench, "arithmeticBench.bang");
benchmark_from_file!(example, bubble_sort, "bubbleSort.bang");
benchmark_from_file!(example, iterative_fibonacci, "iterativeFibonacci.bang");
benchmark_from_file!(example, recursive_fibonacci, "recursiveFibonacci.bang");
benchmark_from_file!(example, syntax, "syntax.bang");
