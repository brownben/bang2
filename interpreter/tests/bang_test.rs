use bang_std::StdContext;
pub use std::rc::Rc;

pub mod bang {
  pub use bang_interpreter::*;
  pub use bang_std::*;
  pub use bang_syntax::*;
}

#[derive(Debug, PartialEq, Eq)]
pub enum RunResult {
  Success,
  RuntimeError,
  CompileError,
  ValidationError,
}

pub fn run(source: &str) -> (RunResult, bang::VM) {
  let chunk = match bang::compile(source, &StdContext) {
    Ok(chunk) => chunk,
    Err(_) => return (RunResult::CompileError, Default::default()),
  };

  if chunk.verify().is_err() {
    return (RunResult::ValidationError, Default::default());
  }

  let mut vm = bang::VM::new(&bang::StdContext);
  match vm.run(&chunk) {
    Ok(_) => (RunResult::Success, vm),
    Err(_) => (RunResult::RuntimeError, Default::default()),
  }
}

#[macro_export]
macro_rules! bang_test {
  ($name:ident $code:literal $( $var:ident == $expected:literal)*) => {
    #[test]
    fn $name(){
      let (result, vm) = run($code);
      assert_eq!(result, RunResult::Success);

      $(
        {
          let variable = vm.get_global(stringify!($var)).unwrap();
          let expected = bang::Value::from($expected);

          assert!(variable == expected, "Expected {expected}, got {variable}");
        };
      )*
    }
  };

  ($name:ident $code:literal RuntimeError) => {
    #[test]
    fn $name(){
      let (result, _vm) = run($code);
      assert_eq!(result, RunResult::RuntimeError);
    }
  };

  ($name:ident $code:literal CompileError) => {
    #[test]
    fn $name(){
      let (result, _vm) = run($code);
      assert_eq!(result, RunResult::CompileError);
    }
  };
}
