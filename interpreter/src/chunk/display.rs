use super::{Chunk, OpCode};
use std::fmt;

impl fmt::Debug for Chunk {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "          ╭─[Bytecode]")?;

    let mut position: usize = 0;
    let mut last_line_number = 0;

    while position < self.code.len() {
      let line_number = self.get_line_number(position);
      if line_number == last_line_number {
        write!(f, "\n     {position:0>4} │ ")?;
      } else {
        write!(f, "\n{line_number:<4} {position:0>4} │ ")?;
        last_line_number = line_number;
      }

      position = disassemble_instruction(f, self, position)?;
    }
    write!(f, "\n──────────╯")
  }
}

fn disassemble_instruction(
  f: &mut fmt::Formatter<'_>,
  chunk: &Chunk,
  pos: usize,
) -> Result<usize, fmt::Error> {
  let instruction = chunk.get(pos);

  match instruction {
    OpCode::Constant => constant_instruction(f, "Constant", chunk, pos),
    OpCode::ConstantLong => constant_long_instruction(f, "Constant Long", chunk, pos),
    OpCode::Null => simple_instruction(f, "Null", pos),
    OpCode::True => simple_instruction(f, "True", pos),
    OpCode::False => simple_instruction(f, "False", pos),
    OpCode::Add => simple_instruction(f, "Add", pos),
    OpCode::Subtract => simple_instruction(f, "Subtract", pos),
    OpCode::Multiply => simple_instruction(f, "Multiply", pos),
    OpCode::Divide => simple_instruction(f, "Divide", pos),
    OpCode::Negate => simple_instruction(f, "Negate", pos),
    OpCode::Not => simple_instruction(f, "Not", pos),
    OpCode::Equal => simple_instruction(f, "Equal", pos),
    OpCode::Greater => simple_instruction(f, "Greater", pos),
    OpCode::Less => simple_instruction(f, "Less", pos),
    OpCode::NotEqual => simple_instruction(f, "Not Equal", pos),
    OpCode::GreaterEqual => simple_instruction(f, "Greater Equal", pos),
    OpCode::LessEqual => simple_instruction(f, "Less Equal", pos),
    OpCode::Pop => simple_instruction(f, "Pop", pos),
    OpCode::Return => simple_instruction(f, "Return", pos),
    OpCode::DefineGlobal => string_instruction(f, "Define Global", chunk, pos),
    OpCode::GetGlobal => string_instruction(f, "Get Global", chunk, pos),
    OpCode::SetGlobal => string_instruction(f, "Set Global", chunk, pos),
    OpCode::Jump => jump_instruction(f, "Jump", 1, chunk, pos),
    OpCode::JumpIfFalse => jump_instruction(f, "Jump If False", 1, chunk, pos),
    OpCode::JumpIfNull => jump_instruction(f, "Jump If Null", 1, chunk, pos),
    OpCode::Loop => jump_instruction(f, "Loop", -1, chunk, pos),
    OpCode::GetLocal => byte_instruction(f, "Get Local", chunk, pos),
    OpCode::GetTemp => byte_instruction(f, "Get Temp", chunk, pos),
    OpCode::SetLocal => byte_instruction(f, "Set Local", chunk, pos),
    OpCode::Call => byte_instruction(f, "Call", chunk, pos),
    OpCode::List => byte_instruction(f, "List", chunk, pos),
    OpCode::ListLong => double_byte_instruction(f, "List Long", chunk, pos),
    OpCode::GetIndex => simple_instruction(f, "Get Index", pos),
    OpCode::SetIndex => simple_instruction(f, "Set Index", pos),
    OpCode::ToString => simple_instruction(f, "To String", pos),
    OpCode::Closure => simple_instruction(f, "Closure", pos),
    OpCode::GetUpvalue => byte_instruction(f, "Get Upvalue", chunk, pos),
    OpCode::SetUpvalue => byte_instruction(f, "Set Upvalue", chunk, pos),
    OpCode::GetAllocated => byte_instruction(f, "Get Upvalue from Local", chunk, pos),
    OpCode::SetAllocated => byte_instruction(f, "Set Upvalue from Local", chunk, pos),
    _ => simple_instruction(f, "Unknown OpCode", pos),
  }
}

fn simple_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  position: usize,
) -> Result<usize, fmt::Error> {
  write!(f, "{name}")?;
  Ok(position + 1)
}

fn constant_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  chunk: &Chunk,
  position: usize,
) -> Result<usize, fmt::Error> {
  let constant_location = chunk.get_value(position + 1);
  let constant = chunk.get_constant(constant_location as usize);

  write!(f, "{name} {constant} ({constant_location})")?;

  Ok(position + 2)
}

fn string_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  chunk: &Chunk,
  position: usize,
) -> Result<usize, fmt::Error> {
  let string_location: usize = chunk.get_value(position + 1).into();
  let string = chunk.get_string(string_location);

  write!(f, "{name} {string} ({string_location})")?;

  Ok(position + 2)
}

fn constant_long_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  chunk: &Chunk,
  position: usize,
) -> Result<usize, fmt::Error> {
  let constant_location: usize = chunk.get_long_value(position + 1).into();
  let constant = chunk.get_constant(constant_location);

  write!(f, "{name} '{constant}' ({constant_location})")?;
  Ok(position + 3)
}

fn byte_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  chunk: &Chunk,
  position: usize,
) -> Result<usize, fmt::Error> {
  let value = chunk.get_value(position + 1);

  write!(f, "{name} {value}")?;
  Ok(position + 2)
}

fn double_byte_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  chunk: &Chunk,
  position: usize,
) -> Result<usize, fmt::Error> {
  let value = chunk.get_long_value(position + 1);

  write!(f, "{name} {value}")?;
  Ok(position + 3)
}

fn jump_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  direction: i8,
  chunk: &Chunk,
  position: usize,
) -> Result<usize, fmt::Error> {
  let jump = chunk.get_long_value(position + 1);

  write!(f, "{name} {}", i32::from(jump) * i32::from(direction))?;
  Ok(position + 3)
}
