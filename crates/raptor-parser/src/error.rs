use crate::lexer::Token;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    LexerError(#[from] crate::lexer::LexerError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(
        "Invalid permission mask\n\nValue must specified as 3 or 4 octal digits (0755, 1777, 644, 640, etc)"
    )]
    InvalidPermissionMask,

    #[error("Unexpected eof")]
    UnexpectedEof,

    #[error("Expected end of line")]
    ExpectedEol,

    #[error("Expected {0}")]
    Expected(&'static str),

    #[error("Expected {} but found {}", .exp.description(), .found.description())]
    Mismatch { exp: Token, found: Token },

    #[error("Expected word")]
    ExpectedWord,
}
