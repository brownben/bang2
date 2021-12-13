pub use bang;
pub use bang::{Chunk, CompileError, Value};

pub use std::collections::HashMap;
pub use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum RunResult {
  Success,
  RuntimeError,
  CompileError,
}

pub fn compile(source: &str) -> Result<Chunk, CompileError> {
  let ast = bang::parse(source)?;
  bang::compile(ast)
}

pub fn run(source: &str) -> (RunResult, HashMap<Rc<str>, Value>) {
  let mut result = RunResult::Success;
  let mut globals = HashMap::new();

  match compile(source) {
    Ok(chunk) => match bang::run(chunk) {
      Ok(vars) => globals = vars,
      Err(_) => result = RunResult::RuntimeError,
    },
    Err(_) => {
      result = RunResult::CompileError;
    }
  };

  (result, globals)
}

#[macro_export]
macro_rules! bang_test {
  ($name:ident $code:literal $( $var:ident == $expected:literal)*) => {
    #[test]
    fn $name(){
      let (result, globals) = run($code);
      assert_eq!(result, RunResult::Success);

      $(
        {
          let variable = globals.get(stringify!($var)).unwrap();
          let expected = Value::from($expected);
          assert!(
            variable.equals(&expected),
            "Expected Variable {} to equal {} but recieved {}",
            stringify!($var),
            expected,
            variable
          );
        };
      )*
    }
  };

  ($name:ident $code:literal RuntimeError) => {
    #[test]
    fn $name(){
      let (result, _globals) = run($code);
      assert_eq!(result, RunResult::RuntimeError);
    }
  };

  ($name:ident $code:literal CompileError) => {
    #[test]
    fn $name(){
      let (result, _globals) = run($code);
      assert_eq!(result, RunResult::CompileError);
    }
  };
}
