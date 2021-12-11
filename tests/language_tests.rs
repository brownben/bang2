use bang::{chunk, compiler, error, parser, vm, Value};

use regex::Regex;

use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

fn is_number(string: &str) -> bool {
  let re = Regex::new(r"^-?([0-9]+.[0-9]+)|([0-9]+)$").unwrap();
  re.is_match(string)
}

fn string_to_value(string: &str) -> Value {
  match string {
    "true" => Value::from(true),
    "false" => Value::from(false),
    "null" => Value::Null,
    num if is_number(num) => Value::from(num.parse::<f64>().unwrap()),
    _ => Value::from(string.to_string()),
  }
}

enum Assertion {
  Variable(String, Value),
  RuntimeError,
  CompileError,
}

fn get_variable_assertion(string: &str) -> Vec<Assertion> {
  let assertion_regex: Regex =
    Regex::new("//assert: (?P<variable>.*?) == (?:\"(.+)\"|(.+))").unwrap();

  string
    .trim()
    .split('\n')
    .map(|assertion| {
      if assertion == "//assert: RuntimeError" {
        Assertion::RuntimeError
      } else if assertion == "//assert: CompileError" {
        Assertion::CompileError
      } else {
        let capture = assertion_regex.captures(assertion).unwrap();
        let variable = capture.name("variable").unwrap().as_str();
        let value = capture
          .get(2)
          .unwrap_or_else(|| capture.get(3).unwrap())
          .as_str()
          .replace("\r", "");

        Assertion::Variable(variable.to_string(), string_to_value(&value))
      }
    })
    .collect()
}

#[derive(Debug, PartialEq)]
enum RunResult {
  Success,
  RuntimeError,
  CompileError,
}

fn compile(source: &str) -> Result<chunk::Chunk, error::CompileError> {
  let ast = parser::parse(source)?;
  compiler::compile(ast)

}

fn run(source: &str) -> (RunResult, HashMap<Rc<str>, Value>) {
  let mut result = RunResult::Success;
  let mut globals = HashMap::new();

  match compile(source) {
    Ok(chunk) => match vm::run(chunk) {
      Ok(vars) => globals = vars,
      Err(_) => result = RunResult::RuntimeError,
    },
    Err(_) => {
      result = RunResult::CompileError;
    }
  };

  (result, globals)
}

fn test_bang_file(file: &str) {
  let test_regex: Regex =
    Regex::new(r"//=(?P<name>.*)\n(?P<code>(?:.*\n)*?)(?P<assertions>(?://assert:.*\n)+)").unwrap();

  for test_case in test_regex.captures_iter(file) {
    let name = test_case.name("name").unwrap().as_str();
    let code = test_case.name("code").unwrap().as_str();
    let assertions = test_case.name("assertions").unwrap().as_str();

    println!("Running test: {}", name);

    let (result, globals) = run(code);

    for assertion in get_variable_assertion(assertions) {
      match assertion {
        Assertion::Variable(variable, expected) => {
          println!("{} {:?}", variable, globals);
          let actual = globals.get(variable.trim()).unwrap();
          assert!(
            actual.equals(&expected),
            "<{}> Expected variable '{}' to equal {:?} but got {:?}",
            name,
            variable,
            expected,
            actual,
          );
        }
        Assertion::RuntimeError => {
          assert_eq!(
            result,
            RunResult::RuntimeError,
            "<{}> Expected Runtime Error",
            name,
          );
        }
        Assertion::CompileError => {
          assert_eq!(
            result,
            RunResult::CompileError,
            "<{}> Expected Compile Error",
            name
          );
        }
      }
    }
  }
}

fn get_bang_test_files() -> Vec<std::path::PathBuf> {
  fs::read_dir("./tests/")
    .unwrap()
    .filter_map(Result::ok)
    .map(|f| f.path())
    .filter(|path| path.to_str().unwrap_or("").ends_with("test.bang"))
    .collect::<Vec<std::path::PathBuf>>()
}

#[test]
fn language_tests() {
  for file_name in get_bang_test_files() {
    println!("\nRunning tests from file: {:?}", file_name);
    if let Ok(file) = fs::read_to_string(file_name.clone()) {
      test_bang_file(&file);
    } else {
      panic!("Could not read test file '{:?}'", file_name);
    }
  }
}
