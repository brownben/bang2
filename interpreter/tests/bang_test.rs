use std::{fmt, mem};
pub mod bang {
  pub use bang_interpreter::*;
  pub use bang_std::*;
  pub use bang_syntax::*;
}

pub enum RunResult<'a> {
  Success(bang::VM<'a>),
  RuntimeError,
  CompileError,
  ValidationError,
}
impl fmt::Debug for RunResult<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Success(_) => write!(f, "Success"),
      Self::RuntimeError => write!(f, "RuntimeError"),
      Self::CompileError => write!(f, "CompileError"),
      Self::ValidationError => write!(f, "ValidationError"),
    }
  }
}
impl PartialEq for RunResult<'_> {
  fn eq(&self, other: &Self) -> bool {
    mem::discriminant(self) == mem::discriminant(other)
  }
}

pub fn run<'a>(source: &str, context: &'a dyn bang::context::Context) -> RunResult<'a> {
  let chunk = match bang::compile(source) {
    Ok(chunk) => chunk,
    Err(_) => return RunResult::CompileError,
  };

  if chunk.verify().is_err() {
    return RunResult::ValidationError;
  }

  let mut vm = bang::VM::new(context);
  match vm.run(&chunk) {
    Ok(_) => RunResult::Success(vm),
    Err(_) => RunResult::RuntimeError,
  }
}

#[macro_export]
macro_rules! bang_test {
  ($name:ident $code:literal $( $var:ident == $expected:literal)*) => {
    #[test]
    fn $name(){
      let context = bang::StdContext::default();
      let result = run($code, &context);

      if let RunResult::Success(vm) = result {
        $(
          {
            let variable = vm.get_global(stringify!($var)).unwrap();
            let expected = bang::Value::from($expected);

            assert!(variable == expected, "Expected {expected}, got {variable}");
          };
        )*
      } else {
        panic!("Execution not successful, {result:?}")
      }
    }
  };

  ($name:ident $code:literal RuntimeError) => {
    #[test]
    fn $name(){
      let context = bang::StdContext::default();
      let result = run($code, &context);

      assert_eq!(result, RunResult::RuntimeError);
    }
  };

  ($name:ident $code:literal CompileError) => {
    #[test]
    fn $name(){
      let context = bang::StdContext::default();
      let result = run($code, &context);

      assert_eq!(result, RunResult::CompileError);
    }
  };
}
