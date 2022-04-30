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
impl From<u8> for OpCode {
  fn from(code: u8) -> Self {
    match code {
      0 => Self::Constant,
      1 => Self::ConstantLong,
      2 => Self::Null,
      3 => Self::True,
      4 => Self::False,
      5 => Self::Add,
      6 => Self::Subtract,
      7 => Self::Multiply,
      8 => Self::Divide,
      9 => Self::Negate,
      10 => Self::Not,
      11 => Self::Equal,
      12 => Self::Greater,
      13 => Self::Less,
      14 => Self::NotEqual,
      15 => Self::GreaterEqual,
      16 => Self::LessEqual,
      17 => Self::Pop,
      18 => Self::DefineGlobal,
      19 => Self::GetGlobal,
      20 => Self::SetGlobal,
      21 => Self::Jump,
      22 => Self::JumpIfFalse,
      23 => Self::JumpIfNull,
      24 => Self::Loop,
      25 => Self::GetLocal,
      26 => Self::SetLocal,
      27 => Self::Return,
      28 => Self::Call,
      29 => Self::List,
      30 => Self::ListLong,
      31 => Self::GetIndex,
      32 => Self::SetIndex,
      _ => Self::Unknown,
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
  lines: LineInfoBuilder,
}
impl Builder {
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
    let value = Value::from(string);
    self.add_constant(value)
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
    u16::from_be_bytes([self.get_value(position), self.get_value(position + 1)])
  }

  pub fn get_constant(&self, pointer: usize) -> Value {
    self.constants[pointer].clone()
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
