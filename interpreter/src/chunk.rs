use crate::value::Value;
use bang_syntax::LineNumber;
use std::rc::Rc;

use std::mem;

#[non_exhaustive]
#[repr(u8)]
pub enum OpCode {
  Constant = 0,
  ConstantLong,
  Null,
  True,
  False,
  Add,
  Subtract,
  Multiply,
  Divide,
  Negate,
  Not,
  Equal,
  Greater,
  Less,
  NotEqual,
  GreaterEqual,
  LessEqual,
  Pop,
  DefineGlobal,
  GetGlobal,
  SetGlobal,
  Jump,
  JumpIfFalse,
  JumpIfNull,
  Loop,
  GetLocal,
  SetLocal,
  Return,
  Call,
  List,
  ListLong,
  GetIndex,
  SetIndex,
  Unknown,
}

type TokensOnLine = u16;
type Line = (LineNumber, TokensOnLine);

struct LineInfoBuilder {
  lines: Vec<Line>,
  last: LineNumber,
  repeated: TokensOnLine,
}
impl LineInfoBuilder {
  fn new() -> Self {
    Self {
      lines: Vec::new(),
      last: 1,
      repeated: 0,
    }
  }

  fn add(&mut self, line: LineNumber) {
    if line == 0 || line == self.last {
      self.repeated += 1;
    } else {
      self.lines.push((self.last, self.repeated));
      self.last = line;
      self.repeated = 1;
    }
  }

  fn finalize(mut self) -> LineInfo {
    if self.repeated > 0 {
      self.lines.push((self.last, self.repeated));
      self.last = 0;
      self.repeated = 0;
    }
    LineInfo { lines: self.lines }
  }
}

#[derive(Clone)]
struct LineInfo {
  lines: Vec<Line>,
}
impl LineInfo {
  fn get(&self, opcode_position: usize) -> LineNumber {
    let length = self.lines.len();
    let mut count = 0;
    let mut line = 0;

    while line < length {
      let (_, repeated) = self.lines[line];
      count += repeated;

      if count as usize > opcode_position {
        break;
      }

      line += 1;
    }

    self.lines[line].0
  }
}

pub struct Builder {
  code: Vec<u8>,
  constants: Vec<Value>,
  strings: Vec<Rc<str>>,
  lines: LineInfoBuilder,
}
impl Builder {
  pub fn new() -> Self {
    Self {
      code: Vec::new(),
      constants: Vec::new(),
      strings: Vec::new(),
      lines: LineInfoBuilder::new(),
    }
  }

  pub fn length(&self) -> usize {
    self.code.len()
  }

  pub fn write_opcode(&mut self, code: OpCode, line: LineNumber) {
    self.write_value(code as u8, line);
  }

  pub fn write_value(&mut self, code: u8, line: LineNumber) {
    self.code.push(code);
    self.lines.add(line);
  }

  pub fn write_long_value(&mut self, code: u16, line: LineNumber) {
    let [a, b] = u16::to_be_bytes(code);
    self.code.push(a);
    self.lines.add(line);
    self.code.push(b);
    self.lines.add(line);
  }

  pub fn add_constant(&mut self, value: Value) -> usize {
    self
      .constants
      .iter()
      .position(|x| value == *x)
      .unwrap_or_else(|| {
        self.constants.push(value);
        self.constants.len() - 1
      })
  }

  pub fn add_constant_string(&mut self, string: &str) -> usize {
    let value = Rc::from(string);
    self
      .strings
      .iter()
      .position(|x| value == *x)
      .unwrap_or_else(|| {
        self.strings.push(value);
        self.strings.len() - 1
      })
  }

  pub fn set_long_value(&mut self, offset: usize, value: u16) {
    let [first_byte, second_byte] = u16::to_be_bytes(value);
    self.code[offset] = first_byte;
    self.code[offset + 1] = second_byte;
  }

  pub fn finalize(self) -> Chunk {
    Chunk {
      code: self.code,
      constants: self.constants,
      strings: self.strings,
      lines: self.lines.finalize(),
    }
  }
}

#[derive(Clone)]
pub struct Chunk {
  pub code: Vec<u8>,
  pub constants: Vec<Value>,
  pub strings: Vec<Rc<str>>,
  lines: LineInfo,
}
impl Chunk {
  #[inline]
  pub fn get(&self, position: usize) -> OpCode {
    // Assume bytecode is valid, so position exists and OpCode is valid
    unsafe { mem::transmute(*self.code.get_unchecked(position)) }
  }

  #[inline]
  pub fn get_value(&self, position: usize) -> u8 {
    // Assume bytecode is valid, so position exists
    unsafe { *self.code.get_unchecked(position) }
  }

  #[inline]
  pub fn get_long_value(&self, position: usize) -> u16 {
    u16::from_be_bytes([self.get_value(position), self.get_value(position + 1)])
  }

  #[inline]
  pub fn get_constant(&self, pointer: usize) -> Value {
    // Assume bytecode is valid, so position exists
    unsafe { self.constants.get_unchecked(pointer) }.clone()
  }

  #[inline]
  pub fn get_string(&self, pointer: usize) -> Rc<str> {
    // Assume bytecode is valid, so position exists
    unsafe { self.strings.get_unchecked(pointer) }.clone()
  }

  pub fn get_line_number(&self, opcode_position: usize) -> LineNumber {
    self.lines.get(opcode_position)
  }

  pub fn merge(&mut self, chunk: &Self) -> usize {
    let offset = self.code.len();
    self.code.extend_from_slice(&chunk.code);
    self.lines.lines.extend_from_slice(&chunk.lines.lines);
    offset
  }
}

#[cfg(test)]
mod test {
  use super::{Chunk, LineInfo};
  use crate::VM;

  #[test]
  fn bytecode_with_invalid_bytecode() {
    let chunk = Chunk {
      code: vec![245],
      constants: Vec::new(),
      strings: Vec::new(),
      lines: LineInfo {
        lines: vec![(1, 1)],
      },
    };
    let mut vm = VM::default();
    let error = vm.run(&chunk).unwrap_err();
    assert_eq!(error.message, "Unknown OpCode");
  }
}
