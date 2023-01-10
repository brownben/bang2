use crate::{
  chunk::{Chunk, OpCode},
  Value, VM,
};

#[derive(Clone)]
pub enum ImportValue {
  Constant(Value),
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
impl<'a> Default for &'a dyn Context {
  fn default() -> Self {
    &Empty
  }
}

pub struct BytecodeFunctionCreator {
  chunk: Chunk,
}
impl Default for BytecodeFunctionCreator {
  fn default() -> Self {
    Self {
      chunk: Chunk::new(),
    }
  }
}
impl BytecodeFunctionCreator {
  pub fn emit_opcode(&mut self, code: OpCode) {
    self.chunk.write_opcode(code, u16::MAX);
  }

  pub fn emit_value(&mut self, value: u8) {
    self.chunk.write_value(value, u16::MAX);
  }

  pub fn emit_long_value(&mut self, value: u16) {
    self.chunk.write_long_value(value, u16::MAX);
  }

  pub fn emit_constant(&mut self, value: Value) {
    let constant_position = self.chunk.add_constant(value);

    if let Ok(constant_position) = u8::try_from(constant_position) {
      self.emit_opcode(OpCode::Constant);
      self.emit_value(constant_position);
    } else {
      unreachable!()
    }
  }

  pub fn finish(self) -> Chunk {
    self.chunk.finalize()
  }
}
