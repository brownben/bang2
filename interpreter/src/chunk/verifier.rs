use super::{Chunk, OpCode};
use crate::value::Object;

pub enum Error {
  UnknownOpcode,
  UnknownConstant,
  UnknownLocation,
  UnknownGlobalName,
  NotEnoughParameters,
}

fn opcode_jump(code: OpCode) -> Result<usize, Error> {
  match code {
    OpCode::Null
    | OpCode::True
    | OpCode::False
    | OpCode::Add
    | OpCode::Subtract
    | OpCode::Multiply
    | OpCode::Divide
    | OpCode::Negate
    | OpCode::Not
    | OpCode::Equal
    | OpCode::Greater
    | OpCode::Less
    | OpCode::NotEqual
    | OpCode::GreaterEqual
    | OpCode::LessEqual
    | OpCode::Pop
    | OpCode::Return
    | OpCode::GetIndex
    | OpCode::SetIndex
    | OpCode::ToString
    | OpCode::Closure => Ok(1),
    OpCode::Constant
    | OpCode::DefineGlobal
    | OpCode::GetGlobal
    | OpCode::SetGlobal
    | OpCode::GetLocal
    | OpCode::GetTemp
    | OpCode::SetLocal
    | OpCode::Call
    | OpCode::List
    | OpCode::ListLong
    | OpCode::GetUpvalue
    | OpCode::SetUpvalue
    | OpCode::GetAllocated
    | OpCode::SetAllocated => Ok(2),
    OpCode::Jump
    | OpCode::JumpIfFalse
    | OpCode::JumpIfNull
    | OpCode::Loop
    | OpCode::ConstantLong => Ok(3),
    _ => Err(Error::UnknownOpcode),
  }
}

fn check_opcodes(chunk: &Chunk) -> Result<(), Error> {
  let mut ip = 0;

  while ip < chunk.code.len() {
    let opcode = chunk.get(ip);
    let next_opcode_jump = opcode_jump(opcode)?;

    if ip + next_opcode_jump > chunk.code.len() {
      return Err(Error::NotEnoughParameters);
    }

    match opcode {
      OpCode::Constant => {
        let constant_location: usize = chunk.get_value(ip + 1).into();
        if constant_location >= chunk.constants.len() {
          return Err(Error::UnknownConstant);
        }
      }
      OpCode::ConstantLong => {
        let constant_location: usize = chunk.get_long_value(ip + 1).into();
        if constant_location >= chunk.constants.len() {
          return Err(Error::UnknownConstant);
        }
      }

      OpCode::JumpIfFalse | OpCode::JumpIfNull | OpCode::Jump => {
        let offset: usize = chunk.get_long_value(ip + 1).into();
        if ip + offset + 1 >= chunk.code.len() {
          return Err(Error::UnknownLocation);
        }
      }
      OpCode::Loop => {
        let offset: usize = chunk.get_long_value(ip + 1).into();
        if offset > ip || ip - offset > chunk.code.len() {
          return Err(Error::UnknownLocation);
        }
      }

      OpCode::DefineGlobal | OpCode::GetGlobal | OpCode::SetGlobal => {
        let name_location: usize = chunk.get_value(ip + 1).into();
        if name_location >= chunk.strings.len() {
          return Err(Error::UnknownGlobalName);
        }
      }

      _ => {}
    }

    ip += next_opcode_jump;
  }

  Ok(())
}

impl Chunk {
  fn functions_point_to_opcode(&self) -> bool {
    let code_length = self.code.len();

    self
      .constants
      .iter()
      .filter_map(|constant| {
        if constant.is_object() && let Object::Function(func) = constant.as_object() {
          Some(func)
        } else {
          None
        }
      })
      .all(|func| func.start < code_length)
  }

  pub fn verify(&self) -> Result<(), Error> {
    if !self.functions_point_to_opcode() {
      return Err(Error::UnknownLocation);
    }

    check_opcodes(self)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::value::Function;

  #[test]
  fn invalid_opcode() {
    let chunk = Chunk {
      code: vec![245],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());

    let chunk = Chunk {
      code: vec![OpCode::Null as u8, 200],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());
  }

  #[test]
  fn unknown_constant() {
    let chunk = Chunk {
      code: vec![OpCode::Constant as u8, 5],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());

    let chunk = Chunk {
      code: vec![OpCode::ConstantLong as u8, 0, 3],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());
  }

  #[test]
  fn jump_locations_exist() {
    let chunk = Chunk {
      code: vec![OpCode::Jump as u8, 0, 5],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());

    let chunk = Chunk {
      code: vec![OpCode::JumpIfFalse as u8, 0, 5],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());

    let chunk = Chunk {
      code: vec![OpCode::JumpIfNull as u8, 0, 5],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());

    let chunk = Chunk {
      code: vec![OpCode::Loop as u8, 0, 3],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());
  }

  #[test]
  fn correct_opcode_parameters() {
    for opcode in [OpCode::List, OpCode::Constant, OpCode::Jump, OpCode::Call] {
      let chunk = Chunk {
        code: vec![opcode as u8],
        ..Default::default()
      };
      assert!(chunk.verify().is_err());
    }
  }

  #[test]
  fn unknown_global() {
    let chunk = Chunk {
      code: vec![OpCode::DefineGlobal as u8, 5],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());

    let chunk = Chunk {
      code: vec![OpCode::SetGlobal as u8, 5],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());

    let chunk = Chunk {
      code: vec![OpCode::GetGlobal as u8, 5],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());
  }

  #[test]
  fn functions_point_to_opcode() {
    let chunk = Chunk {
      constants: vec![Function {
        start: 4,
        ..Default::default()
      }
      .into()],
      ..Default::default()
    };
    assert!(chunk.verify().is_err());
  }
}
