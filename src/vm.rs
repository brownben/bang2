use crate::builtin;
use crate::chunk::{Chunk, OpCode};
use crate::error::RuntimeError;
use crate::value::{Function, Value};

use std::collections::HashMap;
use std::rc::Rc;

macro_rules! runtime_error {
  ($vm:expr, $message:expr, $chunk:expr, $ip:expr) => {{
    $vm.stack.clear();

    let mut line_numbers = vec![$chunk.get_line_number($ip)];

    for frame in $vm.frames.iter().rev() {
      line_numbers.push(frame.function.chunk.get_line_number(frame.ip));
    }

    Err(RuntimeError {
      message: $message.to_string(),
      line_numbers,
    })
  }};
}

macro_rules! numeric_expression {
  ($vm:expr, $token:tt, $chunk:expr, $ip:expr) => {
    numeric_expression!($vm, $token, Number, $chunk, $ip)
  };

  ($vm:expr, $token:tt, $type:tt, $chunk:expr, $ip:expr) => {
    let (right, left) = ($vm.stack.pop().unwrap(),$vm.stack.pop().unwrap());

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

struct CallFrame {
  function: Rc<Function>,
  ip: usize,
  offset: usize,
}

pub struct VM {
  stack: Vec<Value>,
  pub globals: HashMap<Rc<str>, Value>,
  frames: Vec<CallFrame>,
}

impl VM {
  fn new() -> Self {
    Self {
      stack: Vec::new(),
      globals: HashMap::new(),
      frames: Vec::new(),
    }
  }

  fn store_frame(&mut self, function: Rc<Function>, ip: usize, offset: usize) {
    self.frames.push(CallFrame {
      function,
      ip,
      offset,
    });
  }

  fn restore_frame(&mut self) -> CallFrame {
    self.frames.pop().unwrap()
  }

  fn peek(&self) -> &Value {
    self.stack.last().unwrap()
  }

  fn peek_second(&self) -> &Value {
    self.stack.get(self.stack.len() - 2).unwrap()
  }

  fn run(&mut self, function: Rc<Function>) -> Result<(), RuntimeError> {
    let mut function = function;
    let mut ip: usize = 0;
    let mut offset: usize = 0;

    loop {
      let instruction = function.chunk.get(ip);

      match instruction {
        Some(OpCode::Constant) => {
          let constant_location = function.chunk.get_value(ip + 1).unwrap();
          let constant = function
            .chunk
            .get_constant(constant_location as usize)
            .unwrap();
          self.stack.push(constant);
          ip += 2;
        }
        Some(OpCode::ConstantLong) => {
          let constant_location = function.chunk.get_long_value(ip + 1).unwrap() as u16;
          let constant = function
            .chunk
            .get_constant(constant_location as usize)
            .unwrap();
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
            let (right, left) = (self.stack.pop().unwrap(), self.stack.pop().unwrap());

            if !left.is_string() || !right.is_string() {
              break runtime_error!(self, "Both operands must be strings.", function.chunk, ip);
            }

            let concatenated = format!("{}{}", left.get_string_value(), right.get_string_value());
            self.stack.push(Value::from(concatenated));
          } else if first_operand.is_number() {
            numeric_expression!(self, +, function.chunk, ip);
          } else {
            break runtime_error!(
              self,
              "Operands must be two numbers or two strings.",
              function.chunk,
              ip
            );
          }

          ip += 1;
        }
        Some(OpCode::Subtract) => {
          numeric_expression!(self, -, function.chunk, ip);
          ip += 1;
        }
        Some(OpCode::Multiply) => {
          numeric_expression!(self, *, function.chunk, ip);
          ip += 1;
        }
        Some(OpCode::Divide) => {
          numeric_expression!(self, /, function.chunk, ip);
          ip += 1;
        }
        Some(OpCode::Negate) => {
          let value = self.stack.pop().unwrap();
          if let Value::Number(n) = value {
            self.stack.push(Value::from(-n));
          } else {
            break runtime_error!(self, "Operand must be a number.", function.chunk, ip);
          }

          ip += 1;
        }
        Some(OpCode::Not) => {
          let value = self.stack.pop().unwrap();
          self.stack.push(Value::from(value.is_falsy()));
          ip += 1;
        }

        Some(OpCode::Equal) => {
          let (right, left) = (self.stack.pop().unwrap(), self.stack.pop().unwrap());
          self.stack.push(Value::from(left.equals(&right)));
          ip += 1;
        }
        Some(OpCode::Less) => {
          numeric_expression!(self, <, Boolean, function.chunk, ip);
          ip += 1;
        }
        Some(OpCode::Greater) => {
          numeric_expression!(self, >, Boolean, function.chunk, ip);
          ip += 1;
        }

        Some(OpCode::Pop) => {
          self.stack.pop();
          ip += 1;
        }

        Some(OpCode::DefineGlobal) => {
          let name_location = function.chunk.get_value(ip + 1).unwrap();
          let name = function.chunk.get_constant(name_location as usize).unwrap();

          let value = self.stack.pop().unwrap();
          self.globals.insert(name.get_string_value(), value);

          ip += 2;
        }
        Some(OpCode::GetGlobal) => {
          let name_location = function.chunk.get_value(ip + 1).unwrap();
          let name = function.chunk.get_constant(name_location as usize).unwrap();

          let value = self.globals.get(&name.get_string_value());

          if let Some(value) = value {
            self.stack.push(value.clone());
          } else {
            let message = format!("Undefined variable '{}'", name.get_string_value());
            break runtime_error!(self, &message, function.chunk, ip);
          }

          ip += 2;
        }
        Some(OpCode::SetGlobal) => {
          let name_location = function.chunk.get_value(ip + 1).unwrap();
          let name = function.chunk.get_constant(name_location as usize).unwrap();
          let value = self.peek().clone();

          if let std::collections::hash_map::Entry::Occupied(mut entry) =
            self.globals.entry(name.get_string_value())
          {
            entry.insert(value);
          } else {
            let message = format!("Undefined variable '{}'", name.get_string_value());
            break runtime_error!(self, &message, function.chunk, ip);
          }

          ip += 2;
        }
        Some(OpCode::GetLocal) => {
          let slot = function.chunk.get_value(ip + 1).unwrap();
          self.stack.push(self.stack[offset + slot as usize].clone());
          ip += 2;
        }
        Some(OpCode::SetLocal) => {
          let slot = function.chunk.get_value(ip + 1).unwrap();
          self.stack[offset + slot as usize] = self.peek().clone();
          ip += 2;
        }

        Some(OpCode::JumpIfFalse) => {
          let offset = function.chunk.get_long_value(ip + 1).unwrap();
          if self.peek().is_falsy() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        Some(OpCode::JumpIfNull) => {
          let offset = function.chunk.get_long_value(ip + 1).unwrap();
          if self.peek().is_null() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        Some(OpCode::Jump) => {
          let offset = function.chunk.get_long_value(ip + 1).unwrap();
          ip += offset as usize + 1;
        }
        Some(OpCode::Loop) => {
          let offset = function.chunk.get_long_value(ip + 1).unwrap();
          ip -= offset as usize - 1;
        }

        Some(OpCode::Return) => {
          let result = self.stack.pop();

          if self.frames.is_empty() {
            break Ok(());
          }

          while self.stack.len() > offset {
            self.stack.pop();
          }
          self.stack.pop();
          self.stack.push(result.unwrap());

          let frame = self.restore_frame();
          function = frame.function;
          ip = frame.ip;
          offset = frame.offset;
        }
        Some(OpCode::Call) => {
          let arg_count = function.chunk.get_value(ip + 1).unwrap();
          let pos = self.stack.len() - arg_count as usize - 1;
          let callee = self.stack[pos].clone();

          if !callee.is_callable() {
            break runtime_error!(self, "Can only call functions.", function.chunk, ip);
          };

          match callee {
            Value::Function(func) => {
              if arg_count != func.arity {
                let message = format!("Expected {} arguments but got {}.", func.arity, arg_count);
                break runtime_error!(self, message, function.chunk, ip);
              }

              self.store_frame(function.clone(), ip + 2, offset);

              offset = self.stack.len() - arg_count as usize;
              function = func;
              ip = 0;
            }
            Value::NativeFunction(func) => {
              if arg_count != func.arity {
                let message = format!("Expected {} arguments but got {}.", func.arity, arg_count);
                break runtime_error!(self, message, function.chunk, ip);
              }

              let start_of_args = self.stack.len() - arg_count as usize;
              let result = {
                let args = self.stack.drain(start_of_args..);
                (func.func)(args.as_slice())
              };
              self.stack.pop();
              self.stack.push(result);

              ip += 2;
            }
            _ => unreachable!(),
          }
        }
        None => {
          break runtime_error!(self, "Unknown OpCode", function.chunk, ip);
        }
      }

      #[cfg(feature = "debug-stack")]
      self.print_stack(ip);

      if ip >= function.chunk.length() {
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
    println!();
  }
}

pub fn run(chunk: Chunk) -> Result<HashMap<Rc<str>, Value>, RuntimeError> {
  let mut vm = VM::new();
  builtin::define_globals(&mut vm);

  vm.run(Function::script(chunk))?;

  Ok(vm.globals)
}
