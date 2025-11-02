use logos::{Lexer, Logos, Span};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Default)]
pub enum LexerError {
    #[error("Unterminated string literal at position {}", .0.start)]
    UnterminatedString(Span),

    #[default]
    #[error("Lexer error")]
    LexerError,

    #[error("Unsupported string escape: {0:?}")]
    BadEscape(String),
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(error = LexerError)]
pub enum Token {
    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(":")]
    Colon,

    #[token("=")]
    Equals,

    #[token(",")]
    Comma,

    #[token("/")]
    Slash,

    #[token(".")]
    Dot,

    #[token("-")]
    Minus,

    #[token("$")]
    Dollar,

    #[token("@")]
    At,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[regex("[a-zA-Z_][^\\]@/. \n\t\",=:{}\\[-]*")]
    Bareword,

    #[regex("[0-9]+")]
    Number,

    #[token("\n")]
    Newline,

    #[regex(r"#.+\n")]
    Comment,

    #[token("\"", string_callback)]
    String(String),

    #[regex(r"( |\t|\\\n)+")]
    Whitespace,

    Eof,
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(error = LexerError)]
enum StringToken {
    #[token("\"")]
    ExitString,

    #[token(r"\n")]
    EscNewline,

    #[token(r"\t")]
    EscTab,

    #[token("\\\"")]
    EscQuote,

    #[token("\\\\")]
    EscBackslash,

    #[regex(r"\\.", priority = 1)]
    BadEscape,

    #[token("\n")]
    Newline,

    #[regex(r#"[^\\"\n]+"#)]
    Chars,
}

impl Token {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::LBracket => "[",
            Self::RBracket => "]",
            Self::LBrace => "{",
            Self::RBrace => "}",
            Self::Colon => ":",
            Self::Equals => "=",
            Self::Comma => ",",
            Self::Slash => "/",
            Self::Dot => ".",
            Self::Minus => "-",
            Self::Dollar => "$",
            Self::At => "@",
            Self::True => "true",
            Self::False => "false",
            Self::Bareword => "<bareword>",
            Self::Number => "<number>",
            Self::Newline => "\\n",
            Self::Comment => "<comment>",
            Self::String(_) => "<string>",
            Self::Whitespace => "<whitespace>",
            Self::Eof => "<end of file>",
        }
    }

    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::LBracket => "'[' (left bracket)",
            Self::RBracket => "']' (right bracket)",
            Self::LBrace => "'{' (left brace)",
            Self::RBrace => "'}' (right brace)",
            Self::Colon => "':' (colon)",
            Self::Equals => "'=' (equals)",
            Self::Comma => "',' (comma)",
            Self::Slash => "'/' (slash)",
            Self::Dot => "'.' (dot)",
            Self::Minus => "'-' (minus)",
            Self::Dollar => "'$' (dollar)",
            Self::At => "'@' (at)",
            Self::True => "true (keyword)",
            Self::False => "false (keyword)",
            Self::Bareword => "<bareword>",
            Self::Number => "<number>",
            Self::Newline => "\\n (newline)",
            Self::Comment => "<comment>",
            Self::String(_) => "<string>",
            Self::Whitespace => "<whitespace>",
            Self::Eof => "<end of file>",
        }
    }
}

fn string_callback(lex: &mut Lexer<Token>) -> Result<String, LexerError> {
    let mut res = String::new();
    let mut string_lexer = lex.clone().morph();

    loop {
        let Some(token) = string_lexer.next() else {
            break;
        };

        match token? {
            StringToken::ExitString => break,
            StringToken::EscNewline => res.push('\n'),
            StringToken::EscTab => res.push('\t'),
            StringToken::EscQuote => res.push('"'),
            StringToken::EscBackslash => res.push('\\'),
            StringToken::Chars => res.push_str(string_lexer.slice()),
            StringToken::Newline => Err(LexerError::UnterminatedString(string_lexer.span()))?,
            StringToken::BadEscape => Err(LexerError::BadEscape(string_lexer.slice().to_string()))?,
        }
    }

    *lex = string_lexer.morph();

    Ok(res)
}

#[cfg(test)]
mod tests {
    use logos::Logos;

    use crate::lexer::{LexerError, Token};

    macro_rules! next_ok {
        ($lexer:expr, $match:pat $(if $guard:expr)?) => {
            assert!(matches!($lexer.next().unwrap().unwrap(), $match $(if $guard)?));
        };
    }

    macro_rules! next_err {
        ($lexer:expr, $match:pat $(if $guard:expr)? ) => {
            assert!(matches!($lexer.next().unwrap().unwrap_err(), $match $(if $guard)?));
        };
    }

    #[test]
    fn string_escapes() {
        let mut lexer = Token::lexer(r#""foo\tbar"#);
        next_ok!(lexer, Token::String(s) if s == "foo\tbar");

        let mut lexer = Token::lexer(r#""foo\nbar"#);
        next_ok!(lexer, Token::String(s) if s == "foo\nbar");

        let mut lexer = Token::lexer(r#""foo\"bar"#);
        next_ok!(lexer, Token::String(s) if s == "foo\"bar");

        let mut lexer = Token::lexer(r#""foo\\bar"#);
        next_ok!(lexer, Token::String(s) if s == "foo\\bar");
    }

    #[test]
    fn string_escape_err() {
        let mut lexer = Token::lexer(r#""foo\xbar"#);
        next_err!(lexer, LexerError::BadEscape(e) if e == "\\x");
    }
}
