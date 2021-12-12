use crate::chunk::{Chunk, OpCode};
use crate::error::RuntimeError;
use crate::value::Value;

use std::collections::HashMap;
use std::rc::Rc;

macro_rules! get {
  ($vm:expr, $value:expr, $chunk:expr, $ip:expr) => {
    match $value {
      Some(value) => value,
      None => break runtime_error!($vm, "Problem accessing stack value", $chunk, $ip),
    }
  };

  (Pop $vm:expr, $chunk:expr, $ip:expr) => {
    get!($vm, $vm.stack.pop(), $chunk, $ip)
  };

  (Constant $vm:expr, $location:expr, $chunk:expr, $ip:expr) => {
    get!($vm, $chunk.get_constant($location as usize), $chunk, $ip)
  };

  (Value $vm:expr, $chunk:expr, $ip:expr) => {
    get!($vm, $chunk.get_value($ip), $chunk, $ip)
  };

  (Long $vm:expr, $chunk:expr, $ip:expr) => {
    get!($vm, $chunk.get_long_value($ip), $chunk, $ip)
  };

  (Two $vm:expr, $chunk:expr, $ip:expr) => {
    (
      get!(Pop $vm, $chunk, $ip),
      get!(Pop $vm, $chunk, $ip),
    )
  };
}

macro_rules! runtime_error {
  ($vm:expr, $message:expr, $chunk:expr, $ip:expr) => {{
    let line_number = $chunk.get_line_number($ip);

    $vm.stack.clear();

    Err(RuntimeError {
      message: $message.to_string(),
      line_number,
    })
  }};
}

macro_rules! numeric_expression {
  ($vm:expr, $token:tt, $chunk:expr, $ip:expr) => {
    numeric_expression!($vm, $token, Number, $chunk, $ip)
  };

  ($vm:expr, $token:tt, $type:tt, $chunk:expr, $ip:expr) => {
    let (right, left) = get!(Two $vm, $chunk, $ip);

    if !left.is_number() || !right.is_number() {
      break runtime_error!($vm, "Both operands must be numbers.", $chunk, $ip);
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
}

impl VM {
  fn new() -> Self {
    Self {
      stack: Vec::new(),
      globals: HashMap::new(),
    }
  }

  fn peek(&self) -> &Value {
    self.stack.last().unwrap()
  }

  fn peek_second(&self) -> &Value {
    self.stack.get(self.stack.len() - 2).unwrap()
  }

  fn run(&mut self, chunk: &Chunk, offset: usize) -> Result<(), RuntimeError> {
    let mut ip: usize = 0;

    loop {
      let instruction = chunk.get(ip);

      match instruction {
        Some(OpCode::Constant) => {
          let constant_location = get!(Value self, chunk, ip + 1);
          let constant = get!(Constant self, constant_location, chunk, ip);
          self.stack.push(constant);
          ip += 2;
        }
        Some(OpCode::ConstantLong) => {
          let constant_location = get!(Long self, chunk, ip + 1) as u16;
          let constant = get!(Constant self, constant_location, chunk, ip);
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
          let first_operand = self.peek_second();
          if first_operand.is_string() {
            let (right, left) = get!(Two self, chunk, ip);

            if !left.is_string() || !right.is_string() {
              break runtime_error!(self, "Both operands must be strings.", chunk, ip);
            }

            let concatenated = format!("{}{}", left.get_string_value(), right.get_string_value());
            self.stack.push(Value::from(concatenated));
          } else if first_operand.is_number() {
            numeric_expression!(self, +, chunk, ip);
          } else {
            break runtime_error!(
              self,
              "Operands must be two numbers or two strings.",
              chunk,
              ip
            );
          }

          ip += 1;
        }
        Some(OpCode::Subtract) => {
          numeric_expression!(self, -, chunk, ip);
          ip += 1;
        }
        Some(OpCode::Multiply) => {
          numeric_expression!(self, *, chunk, ip);
          ip += 1;
        }
        Some(OpCode::Divide) => {
          numeric_expression!(self, /, chunk, ip);
          ip += 1;
        }
        Some(OpCode::Negate) => {
          let value = get!(Pop self, chunk, ip);
          if let Value::Number(n) = value {
            self.stack.push(Value::from(-n))
          } else {
            break runtime_error!(self, "Operand must be a number.", chunk, ip);
          }

          ip += 1;
        }
        Some(OpCode::Not) => {
          let value = get!(Pop self, chunk, ip);
          self.stack.push(Value::from(value.is_falsy()));
          ip += 1;
        }

        Some(OpCode::Equal) => {
          let (right, left) = get!(Two self, chunk, ip);
          self.stack.push(Value::from(left.equals(&right)));
          ip += 1;
        }
        Some(OpCode::Less) => {
          numeric_expression!(self, <, Boolean, chunk, ip);
          ip += 1;
        }
        Some(OpCode::Greater) => {
          numeric_expression!(self, >, Boolean, chunk, ip);
          ip += 1;
        }

        Some(OpCode::Print) => {
          let value = get!(Pop self, chunk, ip);
          println!("{}", value);
          ip += 1;
        }
        Some(OpCode::Pop) => {
          self.stack.pop();
          ip += 1;
        }

        Some(OpCode::DefineGlobal) => {
          let name_location = get!(Value self, chunk, ip + 1);
          let name = get!(Constant self, name_location, chunk, ip);

          let value = get!(Pop self, chunk, ip);
          self.globals.insert(name.get_string_value(), value);

          ip += 2;
        }
        Some(OpCode::GetGlobal) => {
          let name_location = get!(Value self, chunk, ip + 1);
          let name = get!(Constant self, name_location, chunk, ip);

          let value = self.globals.get(&name.get_string_value());

          if let Some(value) = value {
            self.stack.push(value.clone());
          } else {
            let message = format!("Undefined variable '{}'", name.get_string_value());
            break runtime_error!(self, &message, chunk, ip);
          }

          ip += 2;
        }
        Some(OpCode::SetGlobal) => {
          let name_location = get!(Value self, chunk, ip + 1);
          let name = get!(Constant self, name_location, chunk, ip);
          let value = self.peek().clone();

          if let std::collections::hash_map::Entry::Occupied(mut entry) =
            self.globals.entry(name.get_string_value())
          {
            entry.insert(value);
          } else {
            let message = format!("Undefined variable '{}'", name.get_string_value());
            break runtime_error!(self, &message, chunk, ip);
          }

          ip += 2;
        }
        Some(OpCode::GetLocal) => {
          let slot = get!(Value self, chunk, ip + 1);
          self.stack.push(self.stack[offset + slot as usize].clone());
          ip += 2;
        }
        Some(OpCode::SetLocal) => {
          let slot = get!(Value self, chunk, ip + 1);
          self.stack[offset + slot as usize] = self.peek().clone();
          ip += 2;
        }

        Some(OpCode::JumpIfFalse) => {
          let offset = get!(Long self, chunk, ip + 1);
          if self.peek().is_falsy() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        Some(OpCode::JumpIfNull) => {
          let offset = get!(Long self, chunk, ip + 1);
          if self.peek().is_null() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        Some(OpCode::Jump) => {
          let offset = get!(Long self, chunk, ip + 1);
          ip += offset as usize + 1;
        }
        Some(OpCode::Loop) => {
          let offset = get!(Long self, chunk, ip + 1);
          ip -= offset as usize - 1;
        }

        Some(OpCode::Return) => break Ok(()),
        Some(OpCode::Call) => {
          let arg_count = get!(Value self, chunk, ip + 1);

          let pos = self.stack.len() - arg_count as usize - 1;
          let callee = self.stack[offset + pos].clone();

          let function = if callee.is_function() {
            callee.get_function_value().unwrap()
          } else {
            break runtime_error!(self, "Can only call functions.", chunk, ip);
          };

          if arg_count != function.arity {
            let message = format!(
              "Expected {} arguments but got {}.",
              function.arity, arg_count
            );
            break runtime_error!(self, message, chunk, ip);
          }
          self.run(&function.chunk, self.stack.len() - arg_count as usize)?;

          ip += 2;
        }
        None => {
          break runtime_error!(self, "Unknown OpCode", chunk, ip);
        }
      }

      #[cfg(feature = "debug-stack")]
      self.print_stack(ip);

      if ip >= chunk.length() {
        break Ok(());
      }
    }
  }

  #[cfg(feature = "debug-stack")]
  fn print_stack(&self, ip: usize) {
    print!("{:0>4} │ ", ip);
    for item in &self.stack {
      print!("{}, ", item);
    }
    println!("");
  }
}

pub fn run(chunk: Chunk) -> Result<HashMap<Rc<str>, Value>, RuntimeError> {
  let mut vm = VM::new();

  vm.run(&chunk, 0)?;

  Ok(vm.globals)
}
