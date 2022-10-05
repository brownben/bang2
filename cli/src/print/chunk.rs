use bang_interpreter::{Chunk, OpCode};

pub fn print(chunk: &Chunk) {
  println!("          ╭─[Bytecode]");

  let mut position: usize = 0;
  let mut last_line_number = 0;

  while position < chunk.code.len() {
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
    OpCode::Constant => constant_instruction("Constant", chunk, position),
    OpCode::ConstantLong => constant_long_instruction("Constant Long", chunk, position),
    OpCode::Null => simple_instruction("Null", position),
    OpCode::True => simple_instruction("True", position),
    OpCode::False => simple_instruction("False", position),
    OpCode::Add => simple_instruction("Add", position),
    OpCode::Subtract => simple_instruction("Subtract", position),
    OpCode::Multiply => simple_instruction("Multiply", position),
    OpCode::Divide => simple_instruction("Divide", position),
    OpCode::Negate => simple_instruction("Negate", position),
    OpCode::Not => simple_instruction("Not", position),
    OpCode::Equal => simple_instruction("Equal", position),
    OpCode::Greater => simple_instruction("Greater", position),
    OpCode::Less => simple_instruction("Less", position),
    OpCode::NotEqual => simple_instruction("Not Equal", position),
    OpCode::GreaterEqual => simple_instruction("Greater Equal", position),
    OpCode::LessEqual => simple_instruction("Less Equal", position),
    OpCode::Pop => simple_instruction("Pop", position),
    OpCode::Return => simple_instruction("Return", position),
    OpCode::DefineGlobal => string_instruction("Define Global", chunk, position),
    OpCode::GetGlobal => string_instruction("Get Global", chunk, position),
    OpCode::SetGlobal => string_instruction("Set Global", chunk, position),
    OpCode::Jump => jump_instruction("Jump", 1, chunk, position),
    OpCode::JumpIfFalse => jump_instruction("Jump If False", 1, chunk, position),
    OpCode::JumpIfNull => jump_instruction("Jump If Null", 1, chunk, position),
    OpCode::Loop => jump_instruction("Loop", -1, chunk, position),
    OpCode::GetLocal => byte_instruction("Get Local", chunk, position),
    OpCode::SetLocal => byte_instruction("Set Local", chunk, position),
    OpCode::Call => byte_instruction("Call", chunk, position),
    OpCode::List => byte_instruction("List", chunk, position),
    OpCode::ListLong => double_byte_instruction("List Long", chunk, position),
    OpCode::GetIndex => simple_instruction("Get Index", position),
    OpCode::SetIndex => simple_instruction("Set Index", position),
    OpCode::ToString => simple_instruction("To String", position),
    _ => simple_instruction("Unknown OpCode", position),
  }
}

fn simple_instruction(name: &str, position: usize) -> usize {
  println!("{}", name);
  position + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
  let constant_location = chunk.get_value(position + 1);
  let constant = chunk.get_constant(constant_location as usize);

  println!("{} {} ({})", name, constant, constant_location);

  position + 2
}

fn string_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
  let string_location = chunk.get_value(position + 1);
  let string = chunk.get_string(string_location as usize);

  println!("{} {} ({})", name, string, string_location);

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

fn double_byte_instruction(name: &str, chunk: &Chunk, position: usize) -> usize {
  let value = chunk.get_long_value(position + 1);

  println!("{} {}", name, value);
  position + 3
}

fn jump_instruction(name: &str, direction: i8, chunk: &Chunk, position: usize) -> usize {
  let jump = chunk.get_long_value(position + 1);

  println!("{} {}", name, i32::from(jump) * i32::from(direction));
  position + 3
}
