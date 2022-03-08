use crate::{
  builtins,
  chunk::{Chunk, OpCode},
  diagnostic::Diagnostic,
  value::{Function, Value},
};
use std::{collections::HashMap, rc::Rc};

macro_rules! runtime_error {
  ($vm:expr, $message:expr, $chunk:expr, $ip:expr) => {{
    $vm.stack.clear();

    let mut lines = vec![$chunk.get_line_number($ip)];

    for frame in $vm.frames.iter().rev() {
      lines.push(frame.function.chunk.get_line_number(frame.ip));
    }

    Err(Diagnostic {
      title: $message.to_string(),
      message: "".to_string(),
      lines,
    })
  }};
}

macro_rules! numeric_expression {
  ($vm:expr, $token:tt, $chunk:expr, $ip:expr) => {
    numeric_expression!($vm, $token, Number, $chunk, $ip)
  };

  ($vm:expr, $token:tt, $type:tt, $chunk:expr, $ip:expr) => {
    let (right, left) = ($vm.pop(), $vm.pop());

    match (left, right) {
      (Value::Number(left), Value::Number(right)) => {
        $vm.stack.push(Value::$type(left $token right));
      }
      _ => {
        break runtime_error!($vm, "Both operands must be numbers.", $chunk, $ip);
      }
    }
  };
}

struct CallFrame {
  function: Rc<Function>,
  ip: usize,
  offset: usize,
}

pub type VMGlobals = HashMap<Rc<str>, Value>;

pub struct VM {
  stack: Vec<Value>,
  frames: Vec<CallFrame>,
  globals: VMGlobals,
}

impl VM {
  pub fn new() -> Self {
    Self {
      stack: Vec::with_capacity(32),
      frames: Vec::with_capacity(16),
      globals: HashMap::new(),
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

  fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
  }

  pub fn run(&mut self, chunk: Chunk) -> Result<(), Diagnostic> {
    let mut function = Function::script(chunk);
    let mut ip: usize = 0;
    let mut offset: usize = 0;

    loop {
      let instruction = function.chunk.get(ip);

      match instruction {
        OpCode::Constant => {
          let constant_location = function.chunk.get_value(ip + 1);
          let constant = function.chunk.get_constant(constant_location as usize);
          self.stack.push(constant);
          ip += 2;
        }
        OpCode::ConstantLong => {
          let constant_location = function.chunk.get_long_value(ip + 1) as u16;
          let constant = function.chunk.get_constant(constant_location as usize);
          self.stack.push(constant);
          ip += 3;
        }
        OpCode::Null => {
          self.stack.push(Value::Null);
          ip += 1;
        }
        OpCode::True => {
          self.stack.push(Value::Boolean(true));
          ip += 1;
        }
        OpCode::False => {
          self.stack.push(Value::Boolean(false));
          ip += 1;
        }
        OpCode::Add => {
          let (right, left) = (self.pop(), self.pop());

          match (left, right) {
            (Value::Number(left), Value::Number(right)) => {
              self.stack.push(Value::Number(left + right));
            }
            (Value::String(left), Value::String(right)) => {
              self.stack.push(Value::from([left, right].concat()));
            }
            _ => {
              break runtime_error!(
                self,
                "Operands must be two numbers or two strings.",
                function.chunk,
                ip
              );
            }
          }

          ip += 1;
        }
        OpCode::Subtract => {
          numeric_expression!(self, -, function.chunk, ip);
          ip += 1;
        }
        OpCode::Multiply => {
          numeric_expression!(self, *, function.chunk, ip);
          ip += 1;
        }
        OpCode::Divide => {
          numeric_expression!(self, /, function.chunk, ip);
          ip += 1;
        }
        OpCode::Negate => {
          let value = self.pop();
          if let Value::Number(n) = value {
            self.stack.push(Value::Number(-n));
          } else {
            break runtime_error!(self, "Operand must be a number.", function.chunk, ip);
          }

          ip += 1;
        }
        OpCode::Not => {
          let value = self.pop();
          self.stack.push(Value::Boolean(value.is_falsy()));
          ip += 1;
        }

        OpCode::Equal => {
          let (right, left) = (self.pop(), self.pop());
          self.stack.push(Value::Boolean(left == right));
          ip += 1;
        }
        OpCode::Less => {
          let (right, left) = (self.pop(), self.pop());

          match (left, right) {
            (Value::Number(left), Value::Number(right)) => {
              self.stack.push(Value::Boolean(left < right));
            }
            (Value::String(left), Value::String(right)) => {
              self.stack.push(Value::Boolean(left < right));
            }
            _ => {
              break runtime_error!(
                self,
                "Operands must be two numbers or two strings.",
                function.chunk,
                ip
              );
            }
          }

          ip += 1;
        }
        OpCode::Greater => {
          let (right, left) = (self.pop(), self.pop());

          match (left, right) {
            (Value::Number(left), Value::Number(right)) => {
              self.stack.push(Value::Boolean(left > right));
            }
            (Value::String(left), Value::String(right)) => {
              self.stack.push(Value::Boolean(left > right));
            }
            _ => {
              break runtime_error!(
                self,
                "Operands must be two numbers or two strings.",
                function.chunk,
                ip
              );
            }
          }

          ip += 1;
        }

        OpCode::Pop => {
          self.stack.pop();
          ip += 1;
        }

        OpCode::DefineGlobal => {
          let name_location = function.chunk.get_value(ip + 1);
          let name = function.chunk.get_constant(name_location as usize);

          let value = self.pop();
          self.globals.insert(name.as_str(), value);

          ip += 2;
        }
        OpCode::GetGlobal => {
          let name_location = function.chunk.get_value(ip + 1);
          let name = function.chunk.get_constant(name_location as usize);

          let value = self.globals.get(&name.as_str());

          if let Some(value) = value {
            self.stack.push(value.clone());
          } else {
            break runtime_error!(
              self,
              format!("Undefined variable '{}'", name.as_str()),
              function.chunk,
              ip
            );
          }

          ip += 2;
        }
        OpCode::SetGlobal => {
          let name_location = function.chunk.get_value(ip + 1);
          let name = function.chunk.get_constant(name_location as usize);
          let value = self.peek().clone();

          if let std::collections::hash_map::Entry::Occupied(mut entry) =
            self.globals.entry(name.as_str())
          {
            entry.insert(value);
          } else {
            let message = &format!("Undefined variable '{}'", name.as_str());
            break runtime_error!(self, message, function.chunk, ip);
          }

          ip += 2;
        }
        OpCode::GetLocal => {
          let slot = function.chunk.get_value(ip + 1);
          self.stack.push(self.stack[offset + slot as usize].clone());
          ip += 2;
        }
        OpCode::SetLocal => {
          let slot = function.chunk.get_value(ip + 1);
          self.stack[offset + slot as usize] = self.peek().clone();
          ip += 2;
        }

        OpCode::JumpIfFalse => {
          let offset = function.chunk.get_long_value(ip + 1);
          if self.peek().is_falsy() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        OpCode::JumpIfNull => {
          let offset = function.chunk.get_long_value(ip + 1);
          ip += match self.peek() {
            Value::Null => offset as usize + 1,
            _ => 3,
          };
        }
        OpCode::Jump => {
          let offset = function.chunk.get_long_value(ip + 1);
          ip += offset as usize + 1;
        }
        OpCode::Loop => {
          let offset = function.chunk.get_long_value(ip + 1);
          ip -= offset as usize - 1;
        }

        OpCode::Return => {
          let result = self.stack.pop();

          if self.frames.is_empty() {
            break Ok(());
          }

          self.stack.drain(offset - 1..);
          self.stack.push(result.unwrap());

          let frame = self.restore_frame();
          function = frame.function;
          ip = frame.ip;
          offset = frame.offset;
        }
        OpCode::Call => {
          let arg_count = function.chunk.get_value(ip + 1);
          let pos = self.stack.len() - arg_count as usize - 1;
          let callee = self.stack[pos].clone();

          match callee {
            Value::Function(func) => {
              if arg_count != func.arity {
                let message = &format!("Expected {} arguments but got {}.", func.arity, arg_count);
                break runtime_error!(self, message, function.chunk, ip);
              }

              self.store_frame(function.clone(), ip + 2, offset);

              offset = self.stack.len() - arg_count as usize;
              function = func;
              ip = 0;
            }
            Value::NativeFunction(func) => {
              if arg_count != func.arity {
                let message = &format!("Expected {} arguments but got {}.", func.arity, arg_count);
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
            _ => {
              break runtime_error!(self, "Can only call functions.", function.chunk, ip);
            }
          }
        }
        OpCode::Unknown => {
          break runtime_error!(self, "Unknown OpCode", function.chunk, ip);
        }
      }

      #[cfg(feature = "debug")]
      self.print_stack(ip);
    }
  }

  pub fn define_global(&mut self, name: &str, value: Value) {
    self.globals.insert(Rc::from(name.to_string()), value);
  }

  #[cfg(feature = "debug")]
  fn print_stack(&self, ip: usize) {
    print!("{:0>4} │ ", ip);
    for item in &self.stack {
      print!("{}, ", item);
    }
    println!();
  }
}

pub fn run(chunk: Chunk) -> Result<VMGlobals, Diagnostic> {
  let mut vm = VM::new();
  builtins::define_globals(&mut vm);

  vm.run(chunk)?;

  Ok(vm.globals)
}
