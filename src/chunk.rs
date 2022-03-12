use crate::{tokens::LineNumber, value::Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
  Constant(u8),
  ConstantLong(u16),
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
  DefineGlobal(u8),
  GetGlobal(u8),
  SetGlobal(u8),
  Jump(u16),
  JumpIfFalse(u16),
  JumpIfNull(u16),
  Loop(u16),
  GetLocal(u8),
  SetLocal(u8),
  Return,
  Call(u8),
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
  code: Vec<OpCode>,
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
    self.code.push(code);
    self.lines.add(line);
  }

  pub fn patch_opcode(&mut self, opcode_position: usize, code: OpCode) {
    self.code[opcode_position] = code;
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

  pub fn finalize(&mut self, name: String) -> Chunk {
    self.code.shrink_to_fit();
    self.constants.shrink_to_fit();

    Chunk {
      code: self.code.clone(),
      constants: self.constants.clone(),
      lines: self.lines.finalize(),
      name,
    }
  }
}

#[derive(Clone)]
pub struct Chunk {
  pub name: String,
  pub code: Vec<OpCode>,
  pub constants: Vec<Value>,
  lines: LineInfo,
}
impl Chunk {
  pub fn get(&self, position: usize) -> OpCode {
    self.code[position]
  }

  pub fn get_constant(&self, pointer: usize) -> Value {
    self.constants[pointer].clone()
  }

  pub fn get_line_number(&self, opcode_position: usize) -> LineNumber {
    self.lines.get(opcode_position)
  }
}
