use crate::scanner::Scanner;
use crate::token::TokenType;

pub fn tokens(source: &str) {
  let mut scanner = Scanner::new(source);
  let mut line = 0;

  println!("     ╭─[Tokens]");
  loop {
    let token = scanner.get_token();
    if token.line != line {
      print!("{:>4} │ ", token.line);
      line = token.line;
    } else {
      print!("     │ ");
    }
    println!(
      "{:?} ({})",
      token.token_type,
      token.get_value_from_string(source)
    );

    if token.token_type == TokenType::EndOfFile {
      println!("─────╯");
      break;
    }
  }
}
