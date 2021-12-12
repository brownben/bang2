use crate::chunk::{Chunk, OpCode};
use crate::error::RuntimeError;
use crate::value::Value;

use std::collections::HashMap;
use std::rc::Rc;

macro_rules! get_safe {
  ($vm:expr, $x:expr , $ip:expr ) => {
    match $x {
      Some(value) => value,
      None => break $vm.runtime_error("Problem accessing stack value", $ip),
    }
  };
}

macro_rules! get_two_values {
  ($vm:expr,  $ip:expr ) => {
    (get_pop!($vm, $ip), get_pop!($vm, $ip))
  };
}

macro_rules! get_long_value {
  ( $vm:expr, $ip:expr ) => {
    get_safe!($vm, $vm.chunk.get_long_value($ip), $ip)
  };
}

macro_rules! get_value {
  ( $vm:expr, $ip:expr ) => {
    get_safe!($vm, $vm.chunk.get_value($ip), $ip)
  };
}

macro_rules! get_constant {
  ( $vm:expr, $location:expr, $ip:expr ) => {
    get_safe!($vm, $vm.chunk.get_constant($location as usize), $ip)
  };
}

macro_rules! get_pop {
  ( $vm:expr, $ip:expr ) => {
    get_safe!($vm, $vm.stack.pop(), $ip)
  };
}

macro_rules! numeric_expression {
  ( $vm:expr, $token:tt, $ip:expr ) => {
    numeric_expression!($vm, $token, Number, $ip)
  };

  ( $vm:expr, $token:tt, $type:tt, $ip:expr ) => {
    let (right, left) = get_two_values!($vm, $ip);

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

struct VM {
  stack: Vec<Value>,
  pub globals: HashMap<Rc<str>, Value>,
  chunk: Chunk,
}

impl VM {
  fn new(chunk: Chunk) -> Self {
    Self {
      chunk,
      stack: Vec::new(),
      globals: HashMap::new(),
    }
  }

  fn peek(&self) -> &Value {
    self.stack.last().unwrap()
  }

  fn peek_2(&self) -> &Value {
    self.stack.get(self.stack.len() - 2).unwrap()
  }

  fn run(&mut self) -> Result<(), RuntimeError> {
    let mut ip = 0;

    loop {
      #[cfg(feature = "debug-stack")]
      println!("Stack={:?}", self.stack);

      let instruction = self.chunk.get(ip);

      match instruction {
        Some(OpCode::Constant) => {
          let constant_location = get_value!(self, ip + 1);
          let constant = get_constant!(self, constant_location, ip);
          self.stack.push(constant);
          ip += 2;
        }
        Some(OpCode::ConstantLong) => {
          let constant_location = get_long_value!(self, ip + 1) as u16;
          let constant = get_constant!(self, constant_location, ip);
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
            let (right, left) = get_two_values!(self, ip);

            if !left.is_string() || !right.is_string() {
              break self.runtime_error("Both operands must be strings.", ip);
            }

            let concatenated = format!("{}{}", left.get_string_value(), right.get_string_value());
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
          let value = get_pop!(self, ip);
          match value {
            Value::Number(n) => self.stack.push(Value::from(-n)),
            _ => {
              break self.runtime_error("Operand must be a number.", ip);
            }
          }
          ip += 1;
        }
        Some(OpCode::Not) => {
          let value = get_pop!(self, ip);
          self.stack.push(Value::from(value.is_falsy()));
          ip += 1;
        }

        Some(OpCode::Equal) => {
          let (right, left) = get_two_values!(self, ip);
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
          let value = get_pop!(self, ip);
          println!("{}", value);
          ip += 1;
        }
        Some(OpCode::Pop) => {
          self.stack.pop();
          ip += 1;
        }

        Some(OpCode::DefineGlobal) => {
          let name_location = get_value!(self, ip + 1);
          let name = get_constant!(self, name_location, ip);

          let value = get_pop!(self, ip);
          self.globals.insert(name.get_string_value(), value);

          ip += 2;
        }
        Some(OpCode::GetGlobal) => {
          let name_location = get_value!(self, ip + 1);
          let name = get_constant!(self, name_location, ip);

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
          let name_location = get_value!(self, ip + 1);
          let name = get_constant!(self, name_location, ip);
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
          let slot = get_value!(self, ip + 1);
          self.stack.push(self.stack[slot as usize].clone());
          ip += 2;
        }
        Some(OpCode::SetLocal) => {
          let slot = get_value!(self, ip + 1);
          self.stack[slot as usize] = self.peek().clone();
          ip += 2;
        }

        Some(OpCode::JumpIfFalse) => {
          let offset = get_long_value!(self, ip + 1);
          if self.peek().is_falsy() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        Some(OpCode::JumpIfNull) => {
          let offset = get_long_value!(self, ip + 1);
          if self.peek().is_null() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        Some(OpCode::Jump) => {
          let offset = get_long_value!(self, ip + 1);
          ip += offset as usize + 1;
        }
        Some(OpCode::Loop) => {
          let offset = get_long_value!(self, ip + 1);
          ip -= offset as usize - 1;
        }

        Some(OpCode::Return) => break Ok(()),
        None => {
          break self.runtime_error("Unknown OpCode", ip);
        }
      }

      if ip >= self.chunk.length() {
        break Ok(());
      }
    }
  }

  fn runtime_error(&mut self, message: &str, ip: usize) -> Result<(), RuntimeError> {
    let line_number = self.chunk.get_line_number(ip);

    self.stack.clear();

    Err(RuntimeError {
      message: message.to_string(),
      line_number,
    })
  }
}

pub fn run(chunk: Chunk) -> Result<HashMap<Rc<str>, Value>, RuntimeError> {
  let mut vm = VM::new(chunk);
  vm.run()?;

  Ok(vm.globals)
}
