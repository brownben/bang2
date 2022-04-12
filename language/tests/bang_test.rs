use ahash::AHashMap as HashMap;
pub use bang_language as bang;
pub use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum RunResult {
  Success,
  RuntimeError,
  CompileError,
}

fn compile(source: &str) -> Result<bang::Chunk, bang::Diagnostic> {
  let tokens = bang::tokenize(source);
  let ast = bang::parse(source, &tokens)?;
  bang::compile(source, &ast)
}

pub fn run(source: &str) -> (RunResult, HashMap<Rc<str>, bang::Value>) {
  let chunk = match compile(source) {
    Ok(chunk) => chunk,
    Err(_) => return (RunResult::CompileError, HashMap::new()),
  };

  match bang::run(chunk) {
    Ok(vars) => (RunResult::Success, vars),
    Err(_) => (RunResult::RuntimeError, HashMap::new()),
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
          assert!(variable == &expected);
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
