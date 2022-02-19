use crate::{tokens::LineNumber, value::Value};

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
  Unknown,
}
impl From<u8> for OpCode {
  fn from(code: u8) -> Self {
    match code {
      0 => OpCode::Constant,
      1 => OpCode::ConstantLong,
      2 => OpCode::Null,
      3 => OpCode::True,
      4 => OpCode::False,
      5 => OpCode::Add,
      6 => OpCode::Subtract,
      7 => OpCode::Multiply,
      8 => OpCode::Divide,
      9 => OpCode::Negate,
      10 => OpCode::Not,
      11 => OpCode::Equal,
      12 => OpCode::Greater,
      13 => OpCode::Less,
      14 => OpCode::Pop,
      15 => OpCode::DefineGlobal,
      16 => OpCode::GetGlobal,
      17 => OpCode::SetGlobal,
      18 => OpCode::Jump,
      19 => OpCode::JumpIfFalse,
      20 => OpCode::JumpIfNull,
      21 => OpCode::Loop,
      22 => OpCode::GetLocal,
      23 => OpCode::SetLocal,
      24 => OpCode::Return,
      25 => OpCode::Call,
      _ => OpCode::Unknown,
    }
  }
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
    if length == 0 {
      0
    } else {
      self.lines[line].0
    }
  }
}

pub struct ChunkBuilder {
  code: Vec<u8>,
  constants: Vec<Value>,
  lines: LineInfoBuilder,
}
impl ChunkBuilder {
  pub fn new() -> Self {
    Self {
      code: Vec::new(),
      constants: Vec::new(),
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
    self.code.push((code >> 8) as u8);
    self.lines.add(line);
    self.code.push(code as u8);
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

#[derive(Clone)]
pub struct Chunk {
  pub code: Vec<u8>,
  pub constants: Vec<Value>,
  lines: LineInfo,
}
impl Chunk {
  pub fn get(&self, position: usize) -> OpCode {
    OpCode::from(self.code[position])
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
