use crate::{
  chunk::{Chunk, OpCode},
  context::Context,
  value::{Index, Object, Value},
};
use ahash::AHashMap as HashMap;
use bang_syntax::LineNumber;
use std::{collections::hash_map, error, fmt, rc::Rc};

#[derive(Debug)]
pub struct RuntimeError {
  pub message: String,
  pub lines: Vec<LineNumber>,
}
impl fmt::Display for RuntimeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Runtime Error: {}\nat line {}",
      self.message, self.lines[0]
    )
  }
}
impl error::Error for RuntimeError {}

macro_rules! runtime_error {
  (($vm:expr, $chunk:expr, $ip:expr), $($message:tt)+) => {{
    $vm.stack.clear();

    let mut lines = vec![$chunk.get_line_number($ip)];

    for frame in $vm.frames.iter().rev() {
      lines.push($chunk.get_line_number(frame.ip));
    }

    Err(RuntimeError {
      message: format!($($message)+),
      lines,
    })
  }};
}

macro_rules! numeric_expression {
  ($vm:expr, $token:tt,  $chunk:expr, $ip:expr) => {
    let (right, left) = ($vm.pop(), $vm.pop());

    if left.is_number() && right.is_number() {
      $vm.push(Value::from(left.as_number() $token right.as_number()));
    } else {
      break runtime_error!(($vm, $chunk, $ip), "Both operands must be numbers.");
    }
  };
}

macro_rules! comparison_expression {
  ($vm:expr, $token:tt,  $chunk:expr, $ip:expr) => {
    let (right, left) = ($vm.pop(), $vm.pop());

    if left.is_number() && right.is_number() {
      $vm.push(Value::from(left.as_number() $token right.as_number()));
    } else if left.is_object() && right.is_object() {
      if let Object::String(left) = &*left.as_object() {
        if let Object::String(right) = &*right.as_object(){
          $vm.push(Value::from(left $token right));
        } else {
          break runtime_error!(($vm, $chunk, $ip), "Operands must be two numbers or two strings.");
        }
      } else {
        break runtime_error!(($vm, $chunk, $ip), "Operands must be two numbers or two strings.");
      }
    } else {
      break runtime_error!(($vm, $chunk, $ip), "Operands must be two numbers or two strings.");
    }
  };
}

struct CallFrame {
  ip: usize,
  offset: usize,
}

pub struct VM {
  stack: Vec<Value>,
  frames: Vec<CallFrame>,
  globals: HashMap<Rc<str>, Value>,
}

impl VM {
  pub fn new(context: &dyn Context) -> Self {
    let mut vm = Self::default();
    context.define_globals(&mut vm);
    vm
  }

  #[inline]
  fn store_frame(&mut self, ip: usize, offset: usize) {
    self.frames.push(CallFrame { ip, offset });
  }

  #[inline]
  fn restore_frame(&mut self) -> CallFrame {
    unsafe { self.frames.pop().unwrap_unchecked() }
  }

  #[inline]
  fn peek(&self) -> &Value {
    unsafe { self.stack.last().unwrap_unchecked() }
  }

  #[inline]
  fn pop(&mut self) -> Value {
    unsafe { self.stack.pop().unwrap_unchecked() }
  }

  #[inline]
  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }

  pub fn run(&mut self, chunk: &Chunk) -> Result<(), RuntimeError> {
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
          let constant_location = chunk.get_long_value(ip + 1);
          let constant = chunk.get_constant(constant_location as usize);
          self.push(constant);
          ip += 3;
        }
        OpCode::Null => {
          self.push(Value::NULL);
          ip += 1;
        }
        OpCode::True => {
          self.push(Value::TRUE);
          ip += 1;
        }
        OpCode::False => {
          self.push(Value::FALSE);
          ip += 1;
        }
        OpCode::Add => {
          let (right, left) = (self.pop(), self.pop());

          if left.is_number() && right.is_number() {
            self.push(Value::from(left.as_number() + right.as_number()));
          } else if left.is_object() && right.is_object() {
            if let Object::String(left) = &*left.as_object() {
              if let Object::String(right) = &*right.as_object() {
                self.push(Value::from(format!("{left}{right}")));
              } else {
                break runtime_error!(
                  (self, chunk, ip),
                  "Operands must be two numbers or two strings."
                );
              }
            } else {
              break runtime_error!(
                (self, chunk, ip),
                "Operands must be two numbers or two strings."
              );
            }
          } else {
            break runtime_error!(
              (self, chunk, ip),
              "Operands must be two numbers or two strings."
            );
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
          if value.is_number() {
            self.push(Value::from(-value.as_number()));
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
          self.push(Value::from(value.is_falsy()));
          ip += 1;
        }

        OpCode::Equal => {
          let (right, left) = (self.pop(), self.pop());
          self.push(Value::from(left == right));
          ip += 1;
        }
        OpCode::NotEqual => {
          let (right, left) = (self.pop(), self.pop());
          self.push(Value::from(left != right));
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
          let name = chunk.get_string(name_location as usize);

          let value = self.pop();
          self.globals.insert(name, value);

          ip += 2;
        }
        OpCode::GetGlobal => {
          let name_location = chunk.get_value(ip + 1);
          let name = chunk.get_string(name_location as usize);

          let value = self.globals.get(&name).cloned();

          if let Some(value) = value {
            self.push(value);
          } else {
            break runtime_error!((self, chunk, ip), "Undefined variable '{}'", name);
          }

          ip += 2;
        }
        OpCode::SetGlobal => {
          let name_location = chunk.get_value(ip + 1);
          let name = chunk.get_string(name_location as usize);
          let value = self.peek().clone();

          if let hash_map::Entry::Occupied(mut entry) = self.globals.entry(name.clone()) {
            entry.insert(value);
          } else {
            break runtime_error!((self, chunk, ip), "Undefined variable '{}'", name);
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
          ip += if *self.peek() == Value::NULL {
            offset as usize + 1
          } else {
            3
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
          if self.frames.is_empty() {
            break Ok(());
          }

          let result = self.pop();
          self.stack.drain(offset - 1..);
          self.push(result);

          let frame = self.restore_frame();
          ip = frame.ip;
          offset = frame.offset;
        }
        OpCode::Call => {
          let arg_count = chunk.get_value(ip + 1);
          let pos = self.stack.len() - arg_count as usize - 1;
          let callee = self.stack[pos].clone();

          if !callee.is_object() {
            break runtime_error!((self, chunk, ip), "Can only call functions.");
          }

          if let Object::Function(func) = &*callee.as_object() {
            if !func.arity.check_arg_count(arg_count) {
              break runtime_error!(
                (self, chunk, ip),
                "Expected {} arguments but got {}.",
                func.arity.get_count(),
                arg_count
              );
            }

            // If more arguments than expected, wrap the overflowing ones into a list
            if func.arity.has_varadic_param() {
              let overflow_count = arg_count + 1 - func.arity.get_count();
              let start_of_items = self.stack.len() - overflow_count as usize;
              let items = self.stack.drain(start_of_items..).collect::<Vec<_>>();
              self.push(Value::from(items));
            }

            self.store_frame(ip + 2, offset);
            offset = self.stack.len() - func.arity.get_count() as usize;
            ip = func.start;
          } else if let Object::NativeFunction(func) = &*callee.as_object() {
            if !func.arity.check_arg_count(arg_count) {
              break runtime_error!(
                (self, chunk, ip),
                "Expected {} arguments but got {}.",
                func.arity.get_count(),
                arg_count
              );
            }

            // If more arguments than expected, wrap the overflowing ones into a list
            if func.arity.has_varadic_param() {
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
          } else {
            break runtime_error!((self, chunk, ip), "Can only call functions.");
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

        _ => {
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

  pub fn get_global(&self, name: &str) -> Option<Value> {
    self.globals.get(name).cloned()
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
    Self {
      stack: Vec::with_capacity(64),
      frames: Vec::with_capacity(16),
      globals: HashMap::with_capacity(8),
    }
  }
}
