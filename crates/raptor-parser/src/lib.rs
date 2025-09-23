pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod print;
pub mod util;

pub use error::ParseError;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(pest_consume::Parser)]
#[grammar = "raptorfile.pest"]
pub struct RaptorFileParser;
