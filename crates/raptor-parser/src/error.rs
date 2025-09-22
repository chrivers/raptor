use std::fmt::Display;
use std::sync::Arc;

use camino::Utf8PathBuf;

use lalrpop_util::ParseError as LalrParseError;
use lalrpop_util::lexer::Token;

#[derive(thiserror::Error, Debug)]
pub enum PathParseError {
    #[error("Cannot get parent path from {0:?}")]
    BadPathNoParent(Utf8PathBuf),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseErrorDetails {
    #[error(transparent)]
    PathParse(#[from] PathParseError),

    #[error("Invalid token at position {0}")]
    InvalidToken(usize),

    #[error("Unreconized eof at {0}. Expected {1:?}")]
    UnexpectedEof(usize, Vec<String>),

    #[error("Unreconized token {} at {}..{}. Expected one of {:?}", token.1, token.0, token.2, expected)]
    UnrecognizedToken {
        token: (usize, String, usize),

        expected: Vec<String>,
    },

    #[error("Extra token {} at {}..{}", token.1, token.0, token.2)]
    UnexpectedToken { token: (usize, String, usize) },

    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(thiserror::Error, Debug)]
pub struct ParseError {
    pub path: Arc<Utf8PathBuf>,
    pub details: ParseErrorDetails,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", &self.path, &self.details)
    }
}

impl<'a> From<LalrParseError<usize, Token<'a>, &'a str>> for ParseErrorDetails {
    fn from(value: LalrParseError<usize, Token<'a>, &'a str>) -> Self {
        match value {
            LalrParseError::InvalidToken { location } => Self::InvalidToken(location),
            LalrParseError::UnrecognizedEof { location, expected } => {
                Self::UnexpectedEof(location, expected)
            }
            LalrParseError::UnrecognizedToken {
                token: (l, t, r),
                expected,
            } => Self::UnrecognizedToken {
                token: (l, t.1.to_string(), r),
                expected,
            },
            LalrParseError::ExtraToken { token: (l, t, r) } => Self::UnexpectedToken {
                token: (l, t.to_string(), r),
            },
            LalrParseError::User { error } => Self::ParseError(error.to_string()),
        }
    }
}
