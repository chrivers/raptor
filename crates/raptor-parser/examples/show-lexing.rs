use std::io::Read;
use std::io::Write;

use colored::Colorize;
use log::error;
use logos::Logos;

use raptor_parser::lexer::WordToken;

fn main() -> Result<(), std::io::Error> {
    colog::init();

    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;

    let mut lexer = WordToken::lexer(&buf);

    let mut stdout = std::io::stdout().lock();

    while let Some(token) = lexer.next() {
        match token {
            Ok(WordToken::Bareword(txt)) => write!(stdout, "{}", txt.bright_white())?,
            Ok(WordToken::Newline(txt) | WordToken::Whitespace(txt)) => {
                write!(stdout, "{txt}")?;
            }
            Ok(WordToken::String(txt)) => write!(stdout, "{}", format!("{txt:?}").yellow())?,
            Ok(WordToken::Comment(txt)) => writeln!(stdout, "{}", &txt[..txt.len() - 1].dimmed())?,
            Ok(WordToken::Eof) => break,
            Ok(_) => write!(stdout, "{}", lexer.slice().purple())?,
            Err(err) => error!("Lexer error: {err}"),
        }
    }

    Ok(())
}
