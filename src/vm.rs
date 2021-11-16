use crate::chunk::{Chunk, OpCode};
use crate::compiler::compile;

use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum InterpreterResult {
  OK,
  RuntimeError,
  CompileError,
}

macro_rules! get_safe {
  ( $x:expr  ) => {
    match $x {
      Some(value) => value,
      None => break InterpreterResult::RuntimeError,
    }
  };
}

macro_rules! get_two_values {
  ( $y:expr  ) => {
    (get_safe!($y.pop()), get_safe!($y.pop()))
  };
}

macro_rules! numeric_expression {
  ( $vm:expr, $token:tt, $ip:expr ) => {
    numeric_expression!($vm, $token, Number, $ip)
  };

  ( $vm:expr, $token:tt, $type:tt, $ip:expr ) => {
    let (right, left) = get_two_values!($vm.stack);

    if !left.is_number() || !right.is_number() {
      $vm.runtime_error("Both operands must be numbers.", $ip);
      break InterpreterResult::RuntimeError;
    }

    $vm.stack.push(
      Value::$type(
        left.get_number_value()
          $token
        right.get_number_value()
      )
    );
  };
}

pub struct VM {
  stack: Vec<Value>,
  pub globals: HashMap<String, Value>,
}

impl VM {
  pub fn new() -> Self {
    Self {
      stack: Vec::new(),
      globals: HashMap::new(),
    }
  }

  fn peek(&self) -> &Value {
    self.stack.last().unwrap()
  }

  pub fn run(&mut self, chunk: &Chunk) -> InterpreterResult {
    let mut ip = 0;
    loop {
      #[cfg(feature = "debug-stack")]
      println!("Stack={:?}", self.stack);

      let instruction = chunk.get(ip);

      match instruction {
        Some(OpCode::Constant) => {
          let constant_location = get_safe!(chunk.get_value(ip + 1));
          let constant = get_safe!(chunk.get_constant(constant_location as usize));
          self.stack.push(constant);
          ip += 2;
        }
        Some(OpCode::ConstantLong) => {
          let constant_location = get_safe!(chunk.get_long_value(ip + 1)) as u16;
          let constant = get_safe!(chunk.get_constant(constant_location as usize));
          self.stack.push(constant);
          ip += 3;
        }
        Some(OpCode::Null) => {
          self.stack.push(Value::Null);
          ip += 1;
        }
        Some(OpCode::True) => {
          self.stack.push(Value::from(true));
          ip += 1;
        }
        Some(OpCode::False) => {
          self.stack.push(Value::from(false));
          ip += 1;
        }
        Some(OpCode::Add) => {
          if self.peek().is_number() {
            numeric_expression!(self, +,ip);
          } else {
            let (right, left) = get_two_values!(self.stack);

            if !left.is_string() || !right.is_string() {
              self.runtime_error("Both operands must be strings.", ip);
              break InterpreterResult::RuntimeError;
            }

            let concatenated = left.get_string_value() + &right.get_string_value();
            self.stack.push(Value::from(concatenated));
          }

          ip += 1;
        }
        Some(OpCode::Subtract) => {
          numeric_expression!(self, -, ip);
          ip += 1;
        }
        Some(OpCode::Multiply) => {
          numeric_expression!(self, *,ip);
          ip += 1;
        }
        Some(OpCode::Divide) => {
          numeric_expression!(self, /,ip);
          ip += 1;
        }
        Some(OpCode::Negate) => {
          let value = get_safe!(self.stack.pop());
          match value {
            Value::Number(n) => self.stack.push(Value::from(-n)),
            _ => self.runtime_error("Operand must be a number.", ip),
          }
          ip += 1;
        }
        Some(OpCode::Not) => {
          let value = get_safe!(self.stack.pop());
          self.stack.push(Value::from(value.is_falsy()));
          ip += 1;
        }

        Some(OpCode::Equal) => {
          let (right, left) = get_two_values!(self.stack);
          self.stack.push(Value::from(left.equals(&right)));
          ip += 1;
        }
        Some(OpCode::Less) => {
          numeric_expression!(self, <, Boolean, ip);
          ip += 1;
        }
        Some(OpCode::Greater) => {
          numeric_expression!(self, >, Boolean, ip);
          ip += 1;
        }

        Some(OpCode::Print) => {
          let value = get_safe!(self.stack.pop());
          println!("{}", value);
          ip += 1;
        }
        Some(OpCode::Pop) => {
          self.stack.pop();
          ip += 1;
        }

        Some(OpCode::DefineGlobal) => {
          let name_location = get_safe!(chunk.get_value(ip + 1));
          let name = get_safe!(chunk.get_constant(name_location as usize));

          self
            .globals
            .insert(name.get_string_value(), self.stack.pop().unwrap());

          ip += 2;
        }
        Some(OpCode::GetGlobal) => {
          let name_location = get_safe!(chunk.get_value(ip + 1));
          let name = get_safe!(chunk.get_constant(name_location as usize));

          let value = self.globals.get(&name.get_string_value());

          match value {
            Some(value) => self.stack.push(value.clone()),
            _ => {
              let message = format!("Undefined variable '{}'", name.get_string_value());
              self.runtime_error(&message, ip);
              break InterpreterResult::RuntimeError;
            }
          }

          ip += 2;
        }
        Some(OpCode::SetGlobal) => {
          let name_location = get_safe!(chunk.get_value(ip + 1));
          let name = get_safe!(chunk.get_constant(name_location as usize));

          if self.globals.contains_key(&name.get_string_value()) {
            self
              .globals
              .insert(name.get_string_value(), self.stack.last().unwrap().clone());
          } else {
            let message = format!("Undefined variable '{}'", name.get_string_value());
            self.runtime_error(&message, ip);
            break InterpreterResult::RuntimeError;
          }

          ip += 2;
        }
        Some(OpCode::GetLocal) => {
          let slot = get_safe!(chunk.get_value(ip + 1));
          self.stack.push(self.stack[slot as usize].clone());
          ip += 2;
        }
        Some(OpCode::SetLocal) => {
          let slot = get_safe!(chunk.get_value(ip + 1));
          self.stack[slot as usize] = self.stack.last().unwrap().clone();
          ip += 2;
        }

        Some(OpCode::JumpIfFalse) => {
          let offset = get_safe!(chunk.get_long_value(ip + 1));
          if self.peek().is_falsy() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        Some(OpCode::JumpIfNull) => {
          let offset = get_safe!(chunk.get_long_value(ip + 1));
          if self.peek().is_null() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        Some(OpCode::Jump) => {
          let offset = get_safe!(chunk.get_long_value(ip + 1));
          ip += offset as usize + 1;
        }
        Some(OpCode::Loop) => {
          let offset = get_safe!(chunk.get_long_value(ip + 1));
          ip -= offset as usize - 1;
        }

        Some(OpCode::Return) => {
          break InterpreterResult::OK;
        }
        None => break InterpreterResult::RuntimeError,
      }

      if ip >= chunk.len() {
        break InterpreterResult::OK;
      }
    }
  }

  pub fn interpret(&mut self, source: &str, from: &str) -> InterpreterResult {
    let (chunk, success) = compile(source, from);
    if success {
      self.run(&chunk)
    } else {
      InterpreterResult::CompileError
    }
  }

  fn runtime_error(&mut self, format: &str, _ip: usize) {
    println!("{} {}", red("Error:"), format);

    self.stack.clear();
  }
}

fn red(string: &str) -> String {
  format!("\x1b[0;31m{}\x1b[0m", string)
}
