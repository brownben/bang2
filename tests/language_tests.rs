use bang::{InterpreterResult, Value, VM};
use regex::Regex;
use std::fs;

fn is_number(string: &str) -> bool {
  let re = Regex::new(r"^-?([0-9]+.[0-9]+)|([0-9]+)$").unwrap();
  re.is_match(string)
}

fn string_to_value(string: String) -> Value {
  match string.as_str() {
    "true" => Value::from(true),
    "false" => Value::from(false),
    "null" => Value::Null,
    num @ _ if is_number(num) => Value::from(num.parse::<f64>().unwrap()),
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
    .split("\n")
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

        Assertion::Variable(variable.to_string(), string_to_value(value))
      }
    })
    .collect()
}

fn test_bang_file(file: &str) {
  let test_regex: Regex =
    Regex::new(r"//=(?P<name>.*)\n(?P<code>(?:.*\n)*?)(?P<assertions>(?://assert:.*\n)+)").unwrap();

  for test_case in test_regex.captures_iter(&file) {
    let name = test_case.name("name").unwrap().as_str();
    let code = test_case.name("code").unwrap().as_str();
    let assertions = test_case.name("assertions").unwrap().as_str();

    println!("Running test: {}", name);

    let mut vm = VM::new();
    let result = vm.interpret(code, String::from(name).trim().to_string());

    for assertion in get_variable_assertion(assertions) {
      match assertion {
        Assertion::Variable(variable, expected) => {
          let actual = vm.globals.get(variable.trim()).unwrap();
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
            InterpreterResult::RuntimeError,
            "<{}> Expected Runtime Error",
            name,
          );
        }
        Assertion::CompileError => {
          assert_eq!(
            result,
            InterpreterResult::CompileError,
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
