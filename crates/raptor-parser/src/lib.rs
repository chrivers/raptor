pub mod ast;
pub mod dsl;
pub mod error;
pub mod print;
pub mod util;

pub use error::ParseError;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(pest_consume::Parser)]
#[grammar = "raptorfile.pest"]
pub struct RaptorFileParser;
