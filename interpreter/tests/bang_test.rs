use ahash::AHashMap as HashMap;
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
}

fn compile(source: &str) -> Result<bang::Chunk, bang::Diagnostic> {
  let ast = bang::parse(source)?;
  bang::compile(source, &ast, &bang::StdContext)
}

pub fn run(source: &str) -> (RunResult, HashMap<Rc<str>, bang::Value>) {
  let chunk = match compile(source) {
    Ok(chunk) => chunk,
    Err(_) => return (RunResult::CompileError, HashMap::new()),
  };

  let mut vm = bang::VM::new(&bang::StdContext);
  match vm.run(&chunk) {
    Ok(_) => (RunResult::Success, vm.get_globals()),
    Err(_) => (RunResult::RuntimeError, Default::default()),
  }
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
          let expected = bang::Value::from($expected);

          assert!(variable == &expected, "Expected {expected}, got {variable}");
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
