use crate::{
  builtins,
  chunk::{Chunk, OpCode},
  diagnostic::Diagnostic,
  value::{Index, Value},
};
use ahash::AHashMap as HashMap;
use std::rc::Rc;

macro_rules! runtime_error {
  (($vm:expr, $chunk:expr, $ip:expr), $($message:tt)+) => {{
    $vm.stack.clear();

    let mut lines = vec![$chunk.get_line_number($ip)];

    for frame in $vm.frames.iter().rev() {
      lines.push($chunk.get_line_number(frame.ip));
    }

    Err(Diagnostic {
      title: format!($($message)+),
      message: "".to_string(),
      lines,
    })
  }};
}

macro_rules! numeric_expression {
  ($vm:expr, $token:tt,  $chunk:expr, $ip:expr) => {
    let (right, left) = ($vm.pop(), $vm.pop());

    match (left, right) {
      (Value::Number(left), Value::Number(right)) => {
        $vm.push(Value::Number(left $token right));
      }
      _ => {
        break runtime_error!(($vm, $chunk, $ip), "Both operands must be numbers.");
      }
    }
  };
}

macro_rules! comparison_expression {
  ($vm:expr, $token:tt,  $chunk:expr, $ip:expr) => {
    let (right, left) = ($vm.pop(), $vm.pop());

    match (left, right) {
      (Value::Number(left), Value::Number(right)) => {
        $vm.push(Value::Boolean(left $token right));
      }
      (Value::String(left), Value::String(right)) => {
        $vm.push(Value::Boolean(left $token right));
      }
      _ => {
        break runtime_error!(($vm, $chunk, $ip), "Operands must be two numbers or two strings.");
      }
    }
  };
}

struct CallFrame {
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
    let mut vm = Self {
      stack: Vec::with_capacity(64),
      frames: Vec::with_capacity(16),
      globals: HashMap::new(),
    };

    builtins::define_globals(&mut vm);

    vm
  }

  #[inline]
  fn store_frame(&mut self, ip: usize, offset: usize) {
    self.frames.push(CallFrame { ip, offset });
  }

  #[inline]
  fn restore_frame(&mut self) -> CallFrame {
    self.frames.pop().unwrap()
  }

  #[inline]
  fn peek(&self) -> &Value {
    self.stack.last().unwrap()
  }

  #[inline]
  fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
  }

  #[inline]
  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }

  pub fn run(&mut self, chunk: &Chunk) -> Result<(), Diagnostic> {
    let mut ip: usize = 0;
    let mut offset: usize = 0;

    loop {
      let instruction = chunk.get(ip);

      match instruction {
        OpCode::Constant => {
          let constant_location = chunk.get_value(ip + 1);
          let constant = chunk.get_constant(constant_location as usize);
          self.push(constant);
          ip += 2;
        }
        OpCode::ConstantLong => {
          let constant_location = chunk.get_long_value(ip + 1) as u16;
          let constant = chunk.get_constant(constant_location as usize);
          self.push(constant);
          ip += 3;
        }
        OpCode::Null => {
          self.push(Value::Null);
          ip += 1;
        }
        OpCode::True => {
          self.push(Value::Boolean(true));
          ip += 1;
        }
        OpCode::False => {
          self.push(Value::Boolean(false));
          ip += 1;
        }
        OpCode::Add => {
          let (right, left) = (self.pop(), self.pop());

          match (left, right) {
            (Value::Number(left), Value::Number(right)) => {
              self.push(Value::Number(left + right));
            }
            (Value::String(left), Value::String(right)) => {
              self.push(Value::from([left, right].concat()));
            }
            _ => {
              break runtime_error!(
                (self, chunk, ip),
                "Operands must be two numbers or two strings.",
              );
            }
          }

          ip += 1;
        }
        OpCode::Subtract => {
          numeric_expression!(self, -, chunk, ip);
          ip += 1;
        }
        OpCode::Multiply => {
          numeric_expression!(self, *, chunk, ip);
          ip += 1;
        }
        OpCode::Divide => {
          numeric_expression!(self, /, chunk, ip);
          ip += 1;
        }
        OpCode::Negate => {
          let value = self.pop();
          if let Value::Number(n) = value {
            self.push(Value::Number(-n));
          } else {
            break runtime_error!(
              (self, chunk, ip),
              "Operand must be a number but recieved {}.",
              value.get_type()
            );
          }

          ip += 1;
        }
        OpCode::Not => {
          let value = self.pop();
          self.push(Value::Boolean(value.is_falsy()));
          ip += 1;
        }

        OpCode::Equal => {
          let (right, left) = (self.pop(), self.pop());
          self.push(Value::Boolean(left == right));
          ip += 1;
        }
        OpCode::NotEqual => {
          let (right, left) = (self.pop(), self.pop());
          self.push(Value::Boolean(left != right));
          ip += 1;
        }
        OpCode::Less => {
          comparison_expression!(self, <, chunk, ip);
          ip += 1;
        }
        OpCode::Greater => {
          comparison_expression!(self, >, chunk, ip);
          ip += 1;
        }
        OpCode::LessEqual => {
          comparison_expression!(self, <=, chunk, ip);
          ip += 1;
        }
        OpCode::GreaterEqual => {
          comparison_expression!(self, >=, chunk, ip);
          ip += 1;
        }

        OpCode::Pop => {
          self.stack.pop(); // Don't unwrap as could be empty.
          ip += 1;
        }

        OpCode::DefineGlobal => {
          let name_location = chunk.get_value(ip + 1);
          let name = chunk.get_constant(name_location as usize);

          let value = self.pop();
          self.globals.insert(name.as_str(), value);

          ip += 2;
        }
        OpCode::GetGlobal => {
          let name_location = chunk.get_value(ip + 1);
          let name = chunk.get_constant(name_location as usize);

          let value = self.globals.get(&name.as_str()).cloned();

          if let Some(value) = value {
            self.push(value);
          } else {
            break runtime_error!((self, chunk, ip), "Undefined variable '{}'", name.as_str());
          }

          ip += 2;
        }
        OpCode::SetGlobal => {
          let name_location = chunk.get_value(ip + 1);
          let name = chunk.get_constant(name_location as usize);
          let value = self.peek().clone();

          if let std::collections::hash_map::Entry::Occupied(mut entry) =
            self.globals.entry(name.as_str())
          {
            entry.insert(value);
          } else {
            break runtime_error!((self, chunk, ip), "Undefined variable '{}'", name.as_str());
          }

          ip += 2;
        }
        OpCode::GetLocal => {
          let slot = chunk.get_value(ip + 1);
          self.push(self.stack[offset + slot as usize].clone());
          ip += 2;
        }
        OpCode::SetLocal => {
          let slot = chunk.get_value(ip + 1);
          self.stack[offset + slot as usize] = self.peek().clone();
          ip += 2;
        }

        OpCode::JumpIfFalse => {
          let offset = chunk.get_long_value(ip + 1);
          if self.peek().is_falsy() {
            ip += offset as usize + 1;
          } else {
            ip += 3;
          }
        }
        OpCode::JumpIfNull => {
          let offset = chunk.get_long_value(ip + 1);
          ip += match self.peek() {
            Value::Null => offset as usize + 1,
            _ => 3,
          };
        }
        OpCode::Jump => {
          let offset = chunk.get_long_value(ip + 1);
          ip += offset as usize + 1;
        }
        OpCode::Loop => {
          let offset = chunk.get_long_value(ip + 1);
          ip -= offset as usize - 1;
        }

        OpCode::Return => {
          let result = self.stack.pop(); // Don't unwrap pop as it may be empty

          if self.frames.is_empty() {
            break Ok(());
          }

          self.stack.drain(offset - 1..);
          self.push(result.unwrap());

          let frame = self.restore_frame();
          ip = frame.ip;
          offset = frame.offset;
        }
        OpCode::Call => {
          let arg_count = chunk.get_value(ip + 1);
          let pos = self.stack.len() - arg_count as usize - 1;
          let callee = self.stack[pos].clone();

          match callee {
            Value::Function(func) => {
              if !func.arity.check_arg_count(arg_count) {
                break runtime_error!(
                  (self, chunk, ip),
                  "Expected {} arguments but got {}.",
                  func.arity.get_count(),
                  arg_count
                );
              }

              // If more arguments than expected, wrap the overflowing ones into a list
              if func.arity.has_varadic_param() && func.arity.check_arg_count(arg_count) {
                let overflow_count = arg_count + 1 - func.arity.get_count();
                let start_of_items = self.stack.len() - overflow_count as usize;
                let items = self.stack.drain(start_of_items..).collect::<Vec<_>>();
                self.push(Value::from(items));
              }

              self.store_frame(ip + 2, offset);
              offset = self.stack.len() - func.arity.get_count() as usize;
              ip = func.start;
            }
            Value::NativeFunction(func) => {
              if !func.arity.check_arg_count(arg_count) {
                break runtime_error!(
                  (self, chunk, ip),
                  "Expected {} arguments but got {}.",
                  func.arity.get_count(),
                  arg_count
                );
              }

              // If more arguments than expected, wrap the overflowing ones into a list
              if func.arity.has_varadic_param() && func.arity.check_arg_count(arg_count) {
                let overflow_count = arg_count + 1 - func.arity.get_count();
                let start_of_items = self.stack.len() - overflow_count as usize;
                let items = self.stack.drain(start_of_items..).collect::<Vec<_>>();
                self.push(Value::from(items));
              }

              let start_of_args = self.stack.len() - arg_count as usize;
              let result = {
                let args = self.stack.drain(start_of_args..);
                (func.func)(args.as_slice())
              };
              self.pop();
              self.push(result);

              ip += 2;
            }
            _ => {
              break runtime_error!((self, chunk, ip), "Can only call functions.");
            }
          }
        }

        OpCode::List => {
          let length = chunk.get_value(ip + 1);
          let start_of_items = self.stack.len() - length as usize;

          let items = self.stack.drain(start_of_items..).collect::<Vec<_>>();
          self.push(Value::from(items));

          ip += 2;
        }
        OpCode::ListLong => {
          let length = chunk.get_long_value(ip + 1);
          let start_of_items = self.stack.len() - length as usize;

          let items = self.stack.drain(start_of_items..).collect::<Vec<_>>();
          self.push(Value::from(items));

          ip += 3;
        }

        OpCode::GetIndex => {
          let index = self.pop();
          let item = self.pop();

          match item.get_property(index) {
            Some(value) => self.push(value),
            None => {
              break runtime_error!((self, chunk, ip), "Can't index type {}", item.get_type());
            }
          }

          ip += 1;
        }
        OpCode::SetIndex => {
          let index = self.pop();
          let mut item = self.pop();
          let value = self.peek().clone();

          if !item.set_property(index, value) {
            break runtime_error!(
              (self, chunk, ip),
              "Can't assign to index of type {}",
              item.get_type()
            );
          }

          ip += 1;
        }

        OpCode::Unknown => {
          break runtime_error!((self, chunk, ip), "Unknown OpCode");
        }
      }

      #[cfg(feature = "debug")]
      self.print_stack(ip);
    }
  }

  pub fn define_global(&mut self, name: &str, value: Value) {
    self.globals.insert(Rc::from(name), value);
  }

  #[cfg(feature = "debug")]
  fn print_stack(&self, ip: usize) {
    println!(
      "{ip:0>4} │ {}",
      self
        .stack
        .iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>()
        .join(", ")
    );
  }
}
impl Default for VM {
  fn default() -> Self {
    Self::new()
  }
}

pub fn run(chunk: &Chunk) -> Result<VMGlobals, Diagnostic> {
  let mut vm = VM::new();

  vm.run(chunk)?;

  Ok(vm.globals)
}
