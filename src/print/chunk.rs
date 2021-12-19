use crate::chunk::{Chunk, OpCode};
use crate::value::Value;

pub fn disassemble(chunk: &Chunk, name: &str) {
  disassemble_chunk(chunk, name);
  for constant in &chunk.constants {
    if let Value::Function(function) = constant {
      disassemble(&function.chunk, &function.name);
    }
  }
}

fn disassemble_chunk(chunk: &Chunk, name: &str) {
  println!("          ╭─[Bytecode:{}]", name);

  let mut position: usize = 0;
  let mut last_line_number = 0;

  while position < chunk.length() {
    let line_number = chunk.get_line_number(position);
    if line_number == last_line_number {
      print!("     {:0>4} │ ", position);
    } else {
      print!("{:<4} {:0>4} │ ", line_number, position);
      last_line_number = line_number;
    }

    position = disassemble_instruction(chunk, position);
  }
  println!("──────────╯");
}

fn disassemble_instruction(chunk: &Chunk, position: usize) -> usize {
  let instruction = chunk.get(position);

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

fn simple_instruction(name: &str, position: usize) -> usize {
  println!("{}", name);
  position + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
  let constant_location = chunk.get_value(position + 1);
  let constant = chunk.get_constant(constant_location as usize);

  println!("{} '{}' ({})", name, constant, constant_location);

  position + 2
}

fn constant_long_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
  let constant_location = chunk.get_long_value(position + 1);
  let constant = chunk.get_constant(constant_location as usize);

  println!("{} '{}' ({})", name, constant, constant_location);
  position + 3
}

fn byte_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
  let value = chunk.get_value(position + 1);

  println!("{} {}", name, value);
  position + 2
}

fn jump_instruction(name: &str, direction: i8, chunk: &Chunk, position: usize) -> usize {
  let jump = chunk.get_long_value(position + 1);

  println!("{} {}", name, jump as i32 * direction as i32);
  position + 3
}
