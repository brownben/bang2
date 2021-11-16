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
      break $vm.runtime_error("Both operands must be numbers.", $ip);
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
  chunk: Chunk,
  from: String,
  source: String,
}

impl VM {
  pub fn new() -> Self {
    Self {
      stack: Vec::new(),
      globals: HashMap::new(),
      chunk: Chunk::new(),
      from: String::new(),
      source: String::new(),
    }
  }

  fn peek(&self) -> &Value {
    self.stack.last().unwrap()
  }

  fn peek_2(&self) -> &Value {
    self.stack.get(self.stack.len() - 2).unwrap()
  }

  pub fn run(&mut self) -> InterpreterResult {
    let mut ip = 0;
    loop {
      #[cfg(feature = "debug-stack")]
      println!("Stack={:?}", self.stack);

      let instruction = self.chunk.get(ip);

      match instruction {
        Some(OpCode::Constant) => {
          let constant_location = get_safe!(self.chunk.get_value(ip + 1));
          let constant = get_safe!(self.chunk.get_constant(constant_location as usize));
          self.stack.push(constant);
          ip += 2;
        }
        Some(OpCode::ConstantLong) => {
          let constant_location = get_safe!(self.chunk.get_long_value(ip + 1)) as u16;
          let constant = get_safe!(self.chunk.get_constant(constant_location as usize));
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
          let first_operand = self.peek_2();
          if first_operand.is_string() {
            let (right, left) = get_two_values!(self.stack);

            if !left.is_string() || !right.is_string() {
              break self.runtime_error("Both operands must be strings.", ip);
            }

            let concatenated = left.get_string_value() + &right.get_string_value();
            self.stack.push(Value::from(concatenated));
          } else if first_operand.is_number() {
            numeric_expression!(self, +, ip);
          } else {
            break self.runtime_error("Operands must be two numbers or two strings.", ip);
          }

          ip += 1;
        }
        Some(OpCode::Subtract) => {
          numeric_expression!(self, -, ip);
          ip += 1;
        }
        Some(OpCode::Multiply) => {
          numeric_expression!(self, *, ip);
          ip += 1;
        }
        Some(OpCode::Divide) => {
          numeric_expression!(self, /, ip);
          ip += 1;
        }
        Some(OpCode::Negate) => {
          let value = get_safe!(self.stack.pop());
          match value {
            Value::Number(n) => self.stack.push(Value::from(-n)),
            _ => {
              break self.runtime_error("Operand must be a number.", ip);
            }
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
          let name_location = get_safe!(self.chunk.get_value(ip + 1));
          let name = get_safe!(self.chunk.get_constant(name_location as usize));

          let value = get_safe!(self.stack.pop());
          self.globals.insert(name.get_string_value(), value);

          ip += 2;
        }
        Some(OpCode::GetGlobal) => {
          let name_location = get_safe!(self.chunk.get_value(ip + 1));
          let name = get_safe!(self.chunk.get_constant(name_location as usize));

          let value = self.globals.get(&name.get_string_value());

          if let Some(value) = value {
            self.stack.push(value.clone());
          } else {
            let message = format!("Undefined variable '{}'", name.get_string_value());
            break self.runtime_error(&message, ip);
          }

          ip += 2;
        }
        Some(OpCode::SetGlobal) => {
          let name_location = get_safe!(self.chunk.get_value(ip + 1));
          let name = get_safe!(self.chunk.get_constant(name_location as usize));
          let value = self.peek().clone();

          if let std::collections::hash_map::Entry::Occupied(mut entry) =
            self.globals.entry(name.get_string_value())
          {
            entry.insert(value);
          } else {
            let message = format!("Undefined variable '{}'", name.get_string_value());
            break self.runtime_error(&message, ip);
          }

          ip += 2;
        }
        Some(OpCode::GetLocal) => {
          let slot = get_safe!(self.chunk.get_value(ip + 1));
          self.stack.push(self.stack[slot as usize].clone());
          ip += 2;
        }
        Some(OpCode::SetLocal) => {
          let slot = get_safe!(self.chunk.get_value(ip + 1));
          self.stack[slot as usize] = self.stack.last().unwrap().clone();
          ip += 2;
        }

        Some(OpCode::JumpIfFalse) => {
          let offset = get_safe!(self.chunk.get_long_value(ip + 1));
          if self.peek().is_falsy() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        Some(OpCode::JumpIfNull) => {
          let offset = get_safe!(self.chunk.get_long_value(ip + 1));
          if self.peek().is_null() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        Some(OpCode::Jump) => {
          let offset = get_safe!(self.chunk.get_long_value(ip + 1));
          ip += offset as usize + 1;
        }
        Some(OpCode::Loop) => {
          let offset = get_safe!(self.chunk.get_long_value(ip + 1));
          ip -= offset as usize - 1;
        }

        Some(OpCode::Return) => {
          break InterpreterResult::OK;
        }
        None => {
          break self.runtime_error("Unknown OpCode", ip);
        }
      }

      if ip >= self.chunk.len() {
        break InterpreterResult::OK;
      }
    }
  }

  pub fn interpret(&mut self, source: &str, from: &str) -> InterpreterResult {
    let (chunk, success) = compile(source, from);
    if success {
      self.chunk = chunk;
      self.from = from.to_string();
      self.source = source.to_string();
      self.run()
    } else {
      InterpreterResult::CompileError
    }
  }

  fn runtime_error(&mut self, message: &str, ip: usize) -> InterpreterResult {
    let line_number = self.chunk.get_line_number(ip) as i64;

    eprintln!("{} {}", red("Error:"), message);
    eprintln!("    ╭─[{}]", self.from);

    if line_number > 1 {
      eprintln!("    ·");
    } else {
      eprintln!("    │");
    }

    for i in (line_number - 2)..=line_number {
      if let Some(line) = self.source.lines().nth(i as usize) {
        eprintln!("{:>3} │ {}", i + 1, line);
      }
    }

    if line_number < self.source.lines().count() as i64 - 1 {
      eprintln!("    ·");
    }

    eprintln!("────╯\n");

    self.stack.clear();
    InterpreterResult::RuntimeError
  }
}

impl Default for VM {
  fn default() -> Self {
    Self::new()
  }
}

fn red(string: &str) -> String {
  format!("\x1b[0;31m{}\x1b[0m", string)
}
