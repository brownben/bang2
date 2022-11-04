use bang_syntax::LineNumber;

use crate::{
  chunk::{Builder as ChunkBuilder, Chunk, OpCode},
  value::{Function, Value},
  vm::VM,
};

pub enum ImportValue {
  Constant(Value),
  Bytecode(fn(BytecodeFunctionCreator) -> Chunk, Function),
  ModuleNotFound,
  ItemNotFound,
}
impl ImportValue {
  pub fn unwrap_constant(self) -> Value {
    if let Self::Constant(value) = self {
      value
    } else {
      panic!()
    }
  }
}

pub trait Context {
  fn get_value(&self, module: &str, value: &str) -> ImportValue;
  fn define_globals(&self, vm: &mut VM);
}

pub struct Empty;
impl Context for Empty {
  fn get_value(&self, _: &str, _: &str) -> ImportValue {
    ImportValue::ModuleNotFound
  }
  fn define_globals(&self, _: &mut VM) {}
}

pub struct BytecodeFunctionCreator<'a> {
  base_chunk: &'a mut ChunkBuilder,
  chunk: ChunkBuilder,
  line: LineNumber,
}
impl<'a> BytecodeFunctionCreator<'a> {
  pub fn new(base_chunk: &'a mut ChunkBuilder, line: LineNumber) -> Self {
    Self {
      base_chunk,
      chunk: ChunkBuilder::new(),
      line,
    }
  }
}
impl BytecodeFunctionCreator<'_> {
  pub fn emit_opcode(&mut self, code: OpCode) {
    self.chunk.write_opcode(code, self.line);
  }

  pub fn emit_value(&mut self, value: u8) {
    self.chunk.write_value(value, self.line);
  }

  pub fn emit_long_value(&mut self, value: u16) {
    self.chunk.write_long_value(value, self.line);
  }

  pub fn emit_constant(&mut self, value: Value) {
    let constant_position = self.base_chunk.add_constant(value);

    if let Ok(constant_position) = u8::try_from(constant_position) {
      self.emit_opcode(OpCode::Constant);
      self.emit_value(constant_position);
    } else if let Ok(constant_position) = u16::try_from(constant_position) {
      self.emit_opcode(OpCode::ConstantLong);
      self.emit_long_value(constant_position);
    } else {
      unreachable!()
    }
  }

  pub fn finish(self) -> Chunk {
    self.chunk.finalize()
  }
}
