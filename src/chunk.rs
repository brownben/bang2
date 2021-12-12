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
  Print,
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

fn get_op_code(code: Option<&u8>) -> Option<OpCode> {
  FromPrimitive::from_u8(*code?)
}

type TokensOnLine = u8;
type Line = (LineNumber, TokensOnLine);

#[derive(Debug)]
struct LineInfo {
  lines: Vec<Line>,
  last: LineNumber,
  repeated: TokensOnLine,
}

impl LineInfo {
  fn new() -> Self {
    Self {
      lines: Vec::new(),
      last: 1,
      repeated: 0,
    }
  }

  fn add(&mut self, line: LineNumber) {
    if line == self.last {
      self.repeated += 1;
    } else {
      self.lines.push((self.last, self.repeated));
      self.last = line;
      self.repeated = 1;
    }
  }

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
    } else if line == length {
      self.lines[length - 1].0
    } else {
      self.lines[line].0
    }
  }

  fn finalize(&mut self) {
    if self.repeated > 0 {
      self.lines.push((self.last, self.repeated));
      self.last = 0;
      self.repeated = 0;
      self.lines.shrink_to_fit();
    }
  }
}

#[derive(Debug)]
pub struct Chunk {
  name: String,
  code: Vec<u8>,
  constants: Vec<Value>,
  lines: LineInfo,
}

impl Chunk {
  pub fn new(name: String) -> Self {
    Self {
      name,
      code: Vec::new(),
      constants: Vec::new(),
      lines: LineInfo::new(),
    }
  }

  pub fn write(&mut self, code: OpCode, line: LineNumber) {
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

  pub fn finalize(&mut self) {
    self.code.shrink_to_fit();
    self.constants.shrink_to_fit();
    self.lines.finalize();
  }

  pub fn get_line_number(&self, opcode_position: usize) -> LineNumber {
    self.lines.get(opcode_position)
  }
}

impl Chunk {
  pub fn length(&self) -> usize {
    self.code.len()
  }

  pub fn get(&self, position: usize) -> Option<OpCode> {
    get_op_code(self.code.get(position))
  }

  pub fn get_value(&self, position: usize) -> Option<u8> {
    Some(*self.code.get(position)?)
  }

  pub fn get_long_value(&self, position: usize) -> Option<u16> {
    let first_byte = u16::from(self.get_value(position)?);
    let second_byte = u16::from(self.get_value(position + 1)?);

    Some((first_byte << 8) + second_byte)
  }

  pub fn get_constant(&self, pointer: usize) -> Option<Value> {
    let value = self.constants.get(pointer)?;

    Some(value.clone())
  }
}

#[cfg(feature = "debug-bytecode")]
pub fn disassemble(chunk: &Chunk) {
  disassemble_chunk(chunk);
  for constant in &chunk.constants {
    if constant.is_function() {
      disassemble(&constant.get_function_value().unwrap().chunk);
    }
  }
}

#[cfg(feature = "debug-bytecode")]
pub fn disassemble_chunk(chunk: &Chunk) {
  println!("          ╭─[Bytecode:{}]", chunk.name);

  let mut position: usize = 0;
  let mut last_line_number = 0;

  while position < chunk.length() {
    let line_number = chunk.lines.get(position);
    if line_number == last_line_number {
      print!("     {:0>4} │ ", position);
    } else {
      print!("{:<4} {:0>4} │ ", line_number, position);
      last_line_number = line_number;
    }

    position = disassemble_instruction(chunk, position)
  }
  println!("──────────╯");
}

#[cfg(feature = "debug-bytecode")]
pub fn disassemble_instruction(chunk: &Chunk, position: usize) -> usize {
  let instruction = get_op_code(chunk.code.get(position));

  match instruction {
    Some(OpCode::Constant) => constant_instruction("Constant", chunk, position),
    Some(OpCode::ConstantLong) => constant_long_instruction("Constant Long", chunk, position),
    Some(OpCode::Null) => simple_instruction("Null", position),
    Some(OpCode::True) => simple_instruction("True", position),
    Some(OpCode::False) => simple_instruction("False", position),
    Some(OpCode::Add) => simple_instruction("Add", position),
    Some(OpCode::Subtract) => simple_instruction("Subtract", position),
    Some(OpCode::Multiply) => simple_instruction("Multiply", position),
    Some(OpCode::Divide) => simple_instruction("Divide", position),
    Some(OpCode::Negate) => simple_instruction("Negate", position),
    Some(OpCode::Not) => simple_instruction("Not", position),
    Some(OpCode::Equal) => simple_instruction("Equal", position),
    Some(OpCode::Greater) => simple_instruction("Greater", position),
    Some(OpCode::Less) => simple_instruction("Less", position),
    Some(OpCode::Print) => simple_instruction("Print", position),
    Some(OpCode::Pop) => simple_instruction("Pop", position),
    Some(OpCode::Return) => simple_instruction("Return", position),
    Some(OpCode::DefineGlobal) => constant_instruction("Define Global", chunk, position),
    Some(OpCode::GetGlobal) => constant_instruction("Get Global", chunk, position),
    Some(OpCode::SetGlobal) => constant_instruction("Set Global", chunk, position),
    Some(OpCode::Jump) => jump_instruction("Jump", 1, chunk, position),
    Some(OpCode::JumpIfFalse) => jump_instruction("Jump If False", 1, chunk, position),
    Some(OpCode::JumpIfNull) => jump_instruction("Jump If Null", 1, chunk, position),
    Some(OpCode::Loop) => jump_instruction("Loop", -1, chunk, position),
    Some(OpCode::GetLocal) => byte_instruction("Get Local", chunk, position),
    Some(OpCode::SetLocal) => byte_instruction("Set Local", chunk, position),
    Some(OpCode::Call) => byte_instruction("Call", chunk, position),
    None => simple_instruction("Unknown OpCode", position),
  }
}

#[cfg(feature = "debug-bytecode")]
fn simple_instruction(name: &str, position: usize) -> usize {
  println!("{}", name);
  position + 1
}

#[cfg(feature = "debug-bytecode")]
fn constant_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
  let (constant_location, constant) = match chunk.get_value(position + 1) {
    Some(value) => (value, chunk.get_constant(value as usize)),
    None => (0, None),
  };

  match constant {
    Some(constant) => println!("{} '{}' ({})", name, constant, constant_location),
    None => println!("{} '' ({})", name, constant_location),
  };

  position + 2
}

#[cfg(feature = "debug-bytecode")]
fn constant_long_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
  let (constant_location, constant) = match chunk.get_long_value(position + 1) {
    Some(value) => (value, chunk.get_constant(value as usize)),
    None => (0, None),
  };

  match constant {
    Some(constant) => println!("{} '{}' ({})", name, constant, constant_location),
    None => println!("{} '' ({})", name, constant_location),
  };

  position + 3
}

#[cfg(feature = "debug-bytecode")]
fn byte_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
  let value = match chunk.get_value(position + 1) {
    Some(value) => value,
    None => 0,
  };

  println!("{} {}", name, value);

  position + 2
}

#[cfg(feature = "debug-bytecode")]
fn jump_instruction(name: &str, direction: i8, chunk: &Chunk, position: usize) -> usize {
  let jump = match chunk.get_long_value(position + 1) {
    Some(value) => value,
    _ => 0,
  };

  println!("{} {}", name, jump as i32 * direction as i32);

  position + 3
}
