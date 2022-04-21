use super::remove_carriage_returns;
use bang_language::{Token, TokenType};

pub fn print(source: &str, tokens: &[Token]) {
  let source = source.as_bytes();
  let mut line = 0;

  println!("     ╭─[Tokens]");
  for token in tokens {
    if token.line == line {
      print!("     │ ");
    } else {
      print!("{:>4} │ ", token.line);
      line = token.line;
    }

    let value = if token.ttype == TokenType::EndOfLine {
      ""
    } else {
      token.get_value(source)
    };
    println!("{:?} ({})", token.ttype, remove_carriage_returns(value));
  }
  println!("─────╯");
}
