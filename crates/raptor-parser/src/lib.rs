pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser2;
pub mod print;
pub mod util;

pub use error::ParseError;

pub type ParseResult<T> = Result<T, ParseError>;
