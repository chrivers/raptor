use std::io::Read;
use std::sync::Arc;

use log::error;
use logos::Logos;
use raptor_parser::ParseError;
use raptor_parser::lexer::Token;
use raptor_parser::parser::Parser;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ParseError(#[from] ParseError),
}

type Result<T> = std::result::Result<T, Error>;

fn parse(buf: &str) -> Result<()> {
    let lexer = Token::lexer(buf);
    let mut parser = Parser::new(lexer, Arc::new("<inline>".into()), None);

    for stmt in parser.file()? {
        println!("{}", stmt.inst);
    }

    Ok(())
}

fn main() -> Result<()> {
    colog::init();

    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;

    if let Err(err) = parse(&buf) {
        error!("Parse failed: {err}");
    }
    Ok(())
}
