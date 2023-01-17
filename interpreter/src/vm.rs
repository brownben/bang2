use crate::{
  chunk::OpCode,
  collections::HashMap,
  context::{self, Context, ImportValue},
  value::{
    indexing::{GetResult, Index, SetResult},
    Closure, ClosureKind, Object, Value,
  },
  Chunk,
};
use bang_syntax::LineNumber;
use itertools::Itertools;
use smallvec::SmallVec;
use smartstring::alias::String;
use std::{collections::hash_map, collections::BTreeSet, error, fmt, mem, rc::Rc};

#[derive(Debug)]
pub struct StackTraceLocation {
  pub kind: StackTraceLocationKind,
  pub line: LineNumber,
}

#[derive(Debug)]
pub enum StackTraceLocationKind {
  Function(String),
  Root,
  Builtin,
}

#[derive(Debug)]
pub struct RuntimeError {
  pub message: String,
  pub stack: Vec<StackTraceLocation>,
}
impl fmt::Display for RuntimeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Runtime Error: {}\n at line {}",
      self.message, self.stack[0].line
    )
  }
}
impl error::Error for RuntimeError {}

macro_rules! runtime_error {
  (traceback, $vm:expr, $chunk:expr, $ip:expr, $offset:expr) => {{
    let line = $chunk.get_line_number($ip);

    let kind = if line == u16::MAX {
      StackTraceLocationKind::Builtin
    } else if $offset == 0 {
      StackTraceLocationKind::Root
    } else {
      StackTraceLocationKind::Function(
        $vm.stack[$offset - 1].as_object().get_function_name().clone()
      )
    };

    StackTraceLocation { kind, line }
  }};

  (($vm:expr, $chunk:expr), $($message:tt)+) => {{
    let stack =
      std::iter::once(runtime_error!(traceback, $vm, $chunk, $vm.ip, $vm.offset))
      .chain(
        $vm
          .frames
          .iter()
          .rev()
          .map(|frame| runtime_error!(traceback, $vm, $chunk, frame.ip, frame.offset)),
      )
      .collect();

    $vm.stack.clear();

    Err(RuntimeError {
      message: format!($($message)+).into(),
      stack,
    })
  }};
}

macro_rules! function_arity_check {
  (($vm:expr, $chunk:expr), $arity:expr, $arg_count:expr) => {{
    if !$arity.check_arg_count($arg_count) {
      break runtime_error!(
        ($vm, $chunk),
        "Expected {} arguments but got {}.",
        $arity.get_count(),
        $arg_count
      );
    }
  }};
}

macro_rules! numeric_expression {
  (($vm:expr, $chunk:expr), $token:tt) => {
    let (right, left) = ($vm.pop(), $vm.pop());

    if left.is_number() && right.is_number() {
      $vm.push(Value::from(left.as_number() $token right.as_number()));
    } else {
      break runtime_error!(($vm, $chunk), "Both operands must be numbers.");
    }
  };
}

macro_rules! comparison_expression {
  (($vm:expr, $chunk:expr), $token:tt) => {
    let (right, left) = ($vm.pop(), $vm.pop());

    if left.is_number() && right.is_number() {
      $vm.push(Value::from(left.as_number() $token right.as_number()));
    } else if left.is_object() && right.is_object()
      && let Object::String(left) = left.as_object()
      && let Object::String(right) = right.as_object()
    {
      $vm.push(Value::from(left $token right));
    } else {
      break runtime_error!(($vm, $chunk), "Operands must be two numbers or two strings.");
    }
  };
}

struct CallFrame {
  ip: usize,
  offset: usize,
  chunk: Chunk,
  upvalues: SmallVec<[Value; 4]>,
}

pub struct VM<'context> {
  ip: usize,
  offset: usize,

  stack: Vec<Value>,
  frames: Vec<CallFrame>,
  globals: HashMap<Rc<str>, Value>,
  cyclic: BTreeSet<u64>,

  context: &'context dyn Context,
}

impl<'context> VM<'context> {
  pub fn new(context: &'context dyn Context) -> Self {
    let mut vm = Self {
      context,
      ..Self::default()
    };
    vm.context.define_globals(&mut vm);
    vm
  }
}
impl VM<'_> {
  #[inline]
  fn store_frame(&mut self, chunk: Chunk, upvalues: SmallVec<[Value; 4]>) {
    self.frames.push(CallFrame {
      ip: self.ip + 2,
      offset: self.offset,
      chunk,
      upvalues,
    });
  }

  #[inline]
  fn peek_frame(&self) -> &CallFrame {
    unsafe { self.frames.last().unwrap_unchecked() }
  }

  #[inline]
  fn restore_frame(&mut self) -> Chunk {
    let frame = unsafe { self.frames.pop().unwrap_unchecked() };

    self.ip = frame.ip;
    self.offset = frame.offset;

    frame.chunk
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
    self.ip = 0;
    self.offset = 0;
    let mut chunk: Chunk = chunk.clone();

    loop {
      let instruction = chunk.get(self.ip);

      match instruction {
        OpCode::Constant => {
          let constant_location = chunk.get_value(self.ip + 1);
          let constant = chunk.get_constant(constant_location.into());
          self.push(constant);
          self.ip += 2;
        }
        OpCode::ConstantLong => {
          let constant_location = chunk.get_long_value(self.ip + 1);
          let constant = chunk.get_constant(constant_location.into());
          self.push(constant);
          self.ip += 3;
        }
        OpCode::Null => {
          self.push(Value::NULL);
          self.ip += 1;
        }
        OpCode::True => {
          self.push(Value::TRUE);
          self.ip += 1;
        }
        OpCode::False => {
          self.push(Value::FALSE);
          self.ip += 1;
        }

        OpCode::Add => {
          let (right, left) = (self.pop(), self.pop());

          if left.is_number() && right.is_number() {
            self.push(Value::from(left.as_number() + right.as_number()));
          } else if left.is_object() && right.is_object()
            && let Object::String(left) = left.as_object()
            && let Object::String(right) = right.as_object()
          {
            let mut new = left.clone();
            new.push_str(right);
            self.push(new.into());
          } else {
            break runtime_error!(
              (self, chunk),
              "Operands must be two numbers or two strings."
            );
          }

          self.ip += 1;
        }
        OpCode::Subtract => {
          numeric_expression!((self, chunk), -);
          self.ip += 1;
        }
        OpCode::Multiply => {
          numeric_expression!((self, chunk), *);
          self.ip += 1;
        }
        OpCode::Divide => {
          numeric_expression!((self, chunk), /);
          self.ip += 1;
        }
        OpCode::Remainder => {
          numeric_expression!((self, chunk), %);
          self.ip += 1;
        }

        OpCode::Negate => {
          let value = self.pop();
          if value.is_number() {
            self.push(Value::from(-value.as_number()));
          } else {
            break runtime_error!(
              (self, chunk),
              "Operand must be a number but recieved {}.",
              value.get_type()
            );
          }

          self.ip += 1;
        }
        OpCode::Not => {
          let value = self.pop();
          self.push(value.is_falsy().into());
          self.ip += 1;
        }

        OpCode::Equal => {
          let (right, left) = (self.pop(), self.pop());
          let equals = Value::equals(&left, &right, &mut self.cyclic);
          self.push(equals.into());

          self.cyclic.clear();
          self.ip += 1;
        }
        OpCode::NotEqual => {
          let (right, left) = (self.pop(), self.pop());
          let not_equals = !Value::equals(&left, &right, &mut self.cyclic);
          self.push(not_equals.into());

          self.cyclic.clear();
          self.ip += 1;
        }
        OpCode::Less => {
          comparison_expression!((self, chunk), <);
          self.ip += 1;
        }
        OpCode::Greater => {
          comparison_expression!((self, chunk), >);
          self.ip += 1;
        }
        OpCode::LessEqual => {
          comparison_expression!((self, chunk), <=);
          self.ip += 1;
        }
        OpCode::GreaterEqual => {
          comparison_expression!((self, chunk), >=);
          self.ip += 1;
        }

        OpCode::Pop => {
          self.stack.pop(); // Don't unwrap as could be empty.
          self.ip += 1;
        }

        OpCode::DefineGlobal => {
          let name_location = chunk.get_value(self.ip + 1);
          let name = chunk.get_string(name_location.into());

          let value = self.pop();
          self.globals.insert(name, value);

          self.ip += 2;
        }
        OpCode::GetGlobal => {
          let name_location = chunk.get_value(self.ip + 1);
          let name = chunk.get_string(name_location.into());

          let value = self.globals.get(&name).cloned();

          if let Some(value) = value {
            self.push(value);
          } else {
            break runtime_error!((self, chunk), "Undefined variable '{}'", name);
          }

          self.ip += 2;
        }
        OpCode::SetGlobal => {
          let name_location = chunk.get_value(self.ip + 1);
          let name = chunk.get_string(name_location.into());
          let value = self.peek().clone();

          if let hash_map::Entry::Occupied(mut entry) = self.globals.entry(name.clone()) {
            entry.insert(value);
          } else {
            break runtime_error!((self, chunk), "Undefined variable '{}'", name);
          }

          self.ip += 2;
        }
        OpCode::GetLocal => {
          let slot = chunk.get_value(self.ip + 1);
          self.push(self.stack[self.offset + usize::from(slot)].clone());
          self.ip += 2;
        }
        OpCode::SetLocal => {
          let slot = chunk.get_value(self.ip + 1);
          self.stack[self.offset + usize::from(slot)] = self.peek().clone();
          self.ip += 2;
        }
        OpCode::GetTemp => {
          let slot = chunk.get_value(self.ip + 1);
          self.push(self.stack[self.stack.len() - usize::from(slot) - 1].clone());
          self.ip += 2;
        }

        OpCode::JumpIfFalse => {
          let jump = chunk.get_long_value(self.ip + 1);
          if self.peek().is_falsy() {
            self.ip += usize::from(jump) + 1;
          } else {
            self.ip += 3;
          }
        }
        OpCode::JumpIfNull => {
          let jump = chunk.get_long_value(self.ip + 1);
          self.ip += if *self.peek() == Value::NULL {
            usize::from(jump) + 1
          } else {
            3
          };
        }
        OpCode::Jump => {
          let jump = chunk.get_long_value(self.ip + 1);
          self.ip += usize::from(jump) + 1;
        }
        OpCode::Loop => {
          let jump = chunk.get_long_value(self.ip + 1);
          self.ip -= usize::from(jump) - 1;
        }

        OpCode::Return => {
          if self.frames.is_empty() {
            break Ok(());
          }

          let result = self.pop();
          self.stack.drain(self.offset - 1..);
          self.push(result);

          chunk = self.restore_frame();
        }
        OpCode::Call => {
          let arg_count = chunk.get_value(self.ip + 1);
          let pos = self.stack.len() - usize::from(arg_count) - 1;
          let callee = self.stack[pos].clone();

          if !callee.is_object() {
            break runtime_error!((self, chunk), "Can only call functions.");
          }

          match callee.as_object() {
            Object::Function(func) => {
              function_arity_check!((self, chunk), func.arity, arg_count);

              let chunk = mem::replace(&mut chunk, func.chunk.clone());
              self.store_frame(chunk, SmallVec::new());

              self.ip = 0;
              self.offset = self.stack.len() - func.arity.get_count();
            }
            Object::Closure(closure) => {
              function_arity_check!((self, chunk), closure.func.arity, arg_count);

              let chunk = mem::replace(&mut chunk, closure.func.chunk.clone());
              self.store_frame(chunk, closure.upvalues.clone());

              self.ip = 0;
              self.offset = self.stack.len() - closure.func.arity.get_count();
            }
            Object::NativeFunction(func) => {
              function_arity_check!((self, chunk), func.arity, arg_count);

              let start_of_args = self.stack.len() - func.arity.get_count();
              let result = {
                let args = self.stack.drain(start_of_args..);
                (func.func)(args.as_slice())
              };
              self.pop();
              self.push(result);

              self.ip += 2;
            }
            _ => {
              break runtime_error!((self, chunk), "Can only call functions.");
            }
          }
        }

        OpCode::List => {
          let length = chunk.get_value(self.ip + 1);
          let start_of_items = self.stack.len() - usize::from(length);

          let items = self.stack.drain(start_of_items..).collect::<Vec<_>>();
          self.push(items.into());

          self.ip += 2;
        }
        OpCode::ListLong => {
          let length = chunk.get_long_value(self.ip + 1);
          let start_of_items = self.stack.len() - usize::from(length);

          let items = self.stack.drain(start_of_items..).collect::<Vec<_>>();
          self.push(items.into());

          self.ip += 3;
        }
        OpCode::Dict => {
          let length = chunk.get_value(self.ip + 1);
          let start_of_items = self.stack.len() - usize::from(length) * 2;

          let items = self
            .stack
            .drain(start_of_items..)
            .tuples()
            .collect::<HashMap<_, _>>();

          self.push(items.into());

          self.ip += 2;
        }

        OpCode::GetIndex => {
          let index = self.pop();
          let item = self.pop();

          match item.get_property(&index) {
            GetResult::Found(value) => self.push(value),
            GetResult::NotFound => {
              break runtime_error!((self, chunk), "Index '{}' not found", index);
            }
            GetResult::NotSupported => {
              break runtime_error!((self, chunk), "Can't index type {}", item.get_type());
            }
          }

          self.ip += 1;
        }
        OpCode::SetIndex => {
          let value = self.pop();
          let index = self.pop();
          let mut item = self.pop();

          match item.set_property(&index, value.clone()) {
            SetResult::Set => {}
            SetResult::NotFound => {
              break runtime_error!((self, chunk), "Index '{}' not found", index);
            }
            SetResult::NotSupported => {
              break runtime_error!((self, chunk), "Can't index type {}", item.get_type());
            }
          }

          self.push(value);
          self.ip += 1;
        }

        OpCode::ToString => {
          let value = self.pop();
          self.push(value.to_string().into());

          self.ip += 1;
        }

        OpCode::Closure => {
          let value = self.pop();

          if value.is_object() && let Object::Function(func) = value.as_object() {
            let upvalues = func
              .upvalues
              .iter()
              .map(|(index, closed)| match closed {
                ClosureKind::Open => {
                  let local = &mut self.stack[self.offset + usize::from(*index)];
                  let allocated = local.clone().allocate();
                  *local = allocated.clone();
                  allocated
                }
                ClosureKind::Closed => {
                  self.stack[self.offset + usize::from(*index)].clone()
                }
                ClosureKind::Upvalue => {
                  self.peek_frame().upvalues[usize::from(*index)].clone()
                },
              })
              .collect();

            self.push(Closure::new(func.clone(), upvalues).into());
          } else {
            break runtime_error!((self, chunk), "Can only close over functions");
          }

          self.ip += 1;
        }
        OpCode::GetUpvalue => {
          let upvalue = chunk.get_value(self.ip + 1);
          let address = self.peek_frame().upvalues[usize::from(upvalue)].as_allocated();

          self.push(address.borrow().clone());

          self.ip += 2;
        }
        OpCode::SetUpvalue => {
          let upvalue = chunk.get_value(self.ip + 1);
          let address = self.peek_frame().upvalues[usize::from(upvalue)].as_allocated();

          address.replace(self.peek().clone());

          self.ip += 2;
        }
        OpCode::GetAllocated => {
          let slot = chunk.get_value(self.ip + 1);
          let address = self.stack[self.offset + usize::from(slot)].as_allocated();

          self.push(address.borrow().clone());

          self.ip += 2;
        }
        OpCode::SetAllocated => {
          let slot = chunk.get_value(self.ip + 1);
          let address = self.stack[self.offset + usize::from(slot)].as_allocated();

          address.replace(self.peek().clone());

          self.ip += 2;
        }

        OpCode::Import => {
          let (item, module) = (self.pop(), self.pop());

          match self.context.get_value(module.as_str(), item.as_str()) {
            ImportValue::Constant(value) => {
              self.push(value.clone());
            }
            ImportValue::ModuleNotFound => {
              break runtime_error!((self, chunk), "Module Not Found",);
            }
            ImportValue::ItemNotFound => {
              break runtime_error!((self, chunk), "Item Not Found");
            }
          };

          self.ip += 1;
        }

        _ => {
          break runtime_error!((self, chunk), "Unknown OpCode");
        }
      }

      #[cfg(feature = "debug")]
      self.print_stack(self.ip);
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
        .map(|item| format!("{item:?}"))
        .collect::<Vec<_>>()
        .join(", ")
    );
  }
}
impl Default for VM<'_> {
  fn default() -> Self {
    Self {
      ip: 0,
      offset: 0,

      stack: Vec::with_capacity(64),
      frames: Vec::with_capacity(16),
      globals: HashMap::default(),
      cyclic: BTreeSet::default(),

      context: &context::Empty,
    }
  }
}
