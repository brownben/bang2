use crate::{
  chunk::{Chunk, OpCode},
  collections::String,
  value::Object,
};
use std::{fmt, rc::Rc};

impl fmt::Debug for Chunk {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut functions: Vec<(String, Rc<Self>)> = vec![("Root".into(), self.clone().into())];

    while let Some((name, chunk)) = functions.pop() {
      print_chunk(f, &name, &chunk)?;

      functions.extend(chunk.constants.iter().filter_map(|constant| {
        if constant.is_object() && let Object::Function(func) = constant.as_object() {
          Some((func.name.clone(), func.chunk.clone()))
        } else {
          None
        }
      }));
    }

    Ok(())
  }
}

fn print_chunk(f: &mut fmt::Formatter<'_>, name: &String, chunk: &Chunk) -> fmt::Result {
  write!(f, "          ╭─[Function: {name}]")?;

  let mut position: usize = 0;
  let mut last_line_number = 0;

  while position < chunk.code.len() {
    let line_number = chunk.get_line_number(position);
    if line_number == last_line_number {
      write!(f, "\n     {position:0>4} │ ")?;
    } else {
      write!(f, "\n{line_number:<4} {position:0>4} │ ")?;
      last_line_number = line_number;
    }

    disassemble_instruction(f, chunk, position)?;
    position += chunk.get(position).number_of_bytes().unwrap_or(1);
  }

  writeln!(f, "\n──────────╯")
}

fn disassemble_instruction(f: &mut fmt::Formatter<'_>, chunk: &Chunk, pos: usize) -> fmt::Result {
  let instruction = chunk.get(pos);

  match instruction {
    OpCode::Constant => constant_instruction(f, "Constant", chunk, pos),
    OpCode::ConstantLong => constant_long_instruction(f, "Constant Long", chunk, pos),
    OpCode::Null => write!(f, "Null"),
    OpCode::True => write!(f, "True"),
    OpCode::False => write!(f, "False"),
    OpCode::Add => write!(f, "Add"),
    OpCode::Subtract => write!(f, "Subtract"),
    OpCode::Multiply => write!(f, "Multiply"),
    OpCode::Divide => write!(f, "Divide"),
    OpCode::Negate => write!(f, "Negate"),
    OpCode::Not => write!(f, "Not"),
    OpCode::Equal => write!(f, "Equal"),
    OpCode::Greater => write!(f, "Greater"),
    OpCode::Less => write!(f, "Less"),
    OpCode::NotEqual => write!(f, "Not Equal"),
    OpCode::GreaterEqual => write!(f, "Greater Equal"),
    OpCode::LessEqual => write!(f, "Less Equal"),
    OpCode::Pop => write!(f, "Pop"),
    OpCode::Return => write!(f, "Return"),
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
    OpCode::Dict => byte_instruction(f, "Dict", chunk, pos),
    OpCode::GetIndex => write!(f, "Get Index"),
    OpCode::SetIndex => write!(f, "Set Index"),
    OpCode::ToString => write!(f, "To String"),
    OpCode::Closure => write!(f, "Closure"),
    OpCode::GetUpvalue => byte_instruction(f, "Get Upvalue", chunk, pos),
    OpCode::SetUpvalue => byte_instruction(f, "Set Upvalue", chunk, pos),
    OpCode::GetAllocated => byte_instruction(f, "Get Upvalue from Local", chunk, pos),
    OpCode::SetAllocated => byte_instruction(f, "Set Upvalue from Local", chunk, pos),
    OpCode::Import => write!(f, "Import"),
    _ => write!(f, "Unknown OpCode"),
  }
}

fn constant_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  chunk: &Chunk,
  position: usize,
) -> fmt::Result {
  let constant_location = chunk.get_value(position + 1);
  let constant = chunk.get_constant(constant_location as usize);

  write!(f, "{name} {constant:?} ({constant_location})")
}

fn string_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  chunk: &Chunk,
  position: usize,
) -> fmt::Result {
  let string_location: usize = chunk.get_value(position + 1).into();
  let string = chunk.get_string(string_location);

  write!(f, "{name} {string:?} ({string_location})")
}

fn constant_long_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  chunk: &Chunk,
  position: usize,
) -> fmt::Result {
  let constant_location: usize = chunk.get_long_value(position + 1).into();
  let constant = chunk.get_constant(constant_location);

  write!(f, "{name} '{constant}' ({constant_location})")
}

fn byte_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  chunk: &Chunk,
  position: usize,
) -> fmt::Result {
  let value = chunk.get_value(position + 1);

  write!(f, "{name} {value}")
}

fn double_byte_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  chunk: &Chunk,
  position: usize,
) -> fmt::Result {
  let value = chunk.get_long_value(position + 1);

  write!(f, "{name} {value}")
}

fn jump_instruction(
  f: &mut fmt::Formatter<'_>,
  name: &str,
  direction: i8,
  chunk: &Chunk,
  position: usize,
) -> fmt::Result {
  let jump = chunk.get_long_value(position + 1);

  write!(f, "{name} {}", i32::from(jump) * i32::from(direction))
}
