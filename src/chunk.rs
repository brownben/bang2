use crate::token::LineNumber;
use crate::value::Value;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(FromPrimitive)]
pub enum OpCode {
  Constant,
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
}

fn get_op_code(code: u8) -> Option<OpCode> {
  FromPrimitive::from_u8(code)
}

type TokensOnLine = u16;
type Line = (LineNumber, TokensOnLine);

#[derive(Debug, Clone)]
struct LineInfoCreator {
  lines: Vec<Line>,
  last: LineNumber,
  repeated: TokensOnLine,
}
impl LineInfoCreator {
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

  fn finalize(&mut self) -> LineInfo {
    if self.repeated > 0 {
      self.lines.push((self.last, self.repeated));
      self.last = 0;
      self.repeated = 0;
    }
    self.lines.shrink_to_fit();
    LineInfo {
      lines: self.lines.clone(),
    }
  }
}

#[derive(Debug, Clone)]
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
    if length == 0 {
      0
    } else {
      self.lines[line].0
    }
  }
}

#[derive(Debug, Clone)]
pub struct ChunkCreator {
  code: Vec<u8>,
  constants: Vec<Value>,
  lines: LineInfoCreator,
}
impl ChunkCreator {
  pub fn new() -> Self {
    Self {
      code: Vec::new(),
      constants: Vec::new(),
      lines: LineInfoCreator::new(),
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
    self.code.push((code >> 8) as u8);
    self.code.push(code as u8);
    self.lines.add(line);
    self.lines.add(line);
  }

  pub fn add_constant(&mut self, value: Value) -> usize {
    self
      .constants
      .iter()
      .position(|x| value.equals(x))
      .unwrap_or_else(|| {
        self.constants.push(value);
        self.constants.len() - 1
      })
  }

  pub fn add_constant_string(&mut self, string: String) -> usize {
    let value = Value::from(string);
    self.add_constant(value)
  }

  pub fn set_long_value(&mut self, offset: usize, value: u16) {
    self.code[offset] = (value >> 8) as u8;
    self.code[offset + 1] = value as u8;
  }

  pub fn finalize(&mut self) -> Chunk {
    self.code.shrink_to_fit();
    self.constants.shrink_to_fit();

    Chunk {
      code: self.code.clone(),
      constants: self.constants.clone(),
      lines: self.lines.finalize(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Chunk {
  code: Vec<u8>,
  pub constants: Vec<Value>,
  lines: LineInfo,
}
impl Chunk {
  pub fn length(&self) -> usize {
    self.code.len()
  }

  pub fn get(&self, position: usize) -> Option<OpCode> {
    get_op_code(self.code[position])
  }

  pub fn get_value(&self, position: usize) -> u8 {
    self.code[position]
  }

  pub fn get_long_value(&self, position: usize) -> u16 {
    let first_byte = u16::from(self.get_value(position));
    let second_byte = u16::from(self.get_value(position + 1));

    (first_byte << 8) + second_byte
  }

  pub fn get_constant(&self, pointer: usize) -> Value {
    self.constants[pointer].clone()
  }

  pub fn get_line_number(&self, opcode_position: usize) -> LineNumber {
    self.lines.get(opcode_position)
  }
}
