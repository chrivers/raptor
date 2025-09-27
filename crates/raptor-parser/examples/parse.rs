use std::io::Read;
use std::sync::Arc;

use log::error;
use logos::Logos;
use raptor_parser::ParseResult;
use raptor_parser::lexer::Token;
use raptor_parser::parser::Parser;

fn parse(buf: &str) -> ParseResult<()> {
    let lexer = Token::lexer(buf);
    let mut parser = Parser::new(lexer, Arc::new("<inline>".into()));

    for stmt in parser.file()? {
        println!("{}", stmt.inst);
    }

    Ok(())
}

fn main() -> ParseResult<()> {
    colog::init();

    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;

    if let Err(err) = parse(&buf) {
        error!("Parse failed: {err}");
    }
    Ok(())
}
