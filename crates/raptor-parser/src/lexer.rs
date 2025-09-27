use logos::{Lexer, Logos, Span};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Default)]
pub enum LexerError {
    #[error("Unterminated string literal at position {}", .0.start)]
    UnterminatedString(Span),

    #[default]
    #[error("Lexer error")]
    LexerError,
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(error = LexerError)]
pub enum WordToken {
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

    #[regex("[^\\]/. \n\t\",=:{}\\[]+")]
    Bareword,

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

    #[token("\n")]
    Newline,

    #[regex(r#"[^\\"\n]+"#)]
    Chars,
}

impl WordToken {
    #[must_use]
    pub const fn is_whitespace(&self) -> bool {
        matches!(self, Self::Whitespace)
    }
}

fn string_callback(lex: &mut Lexer<WordToken>) -> Result<String, LexerError> {
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
            StringToken::Chars => res.push_str(string_lexer.slice()),
            StringToken::Newline => Err(LexerError::UnterminatedString(string_lexer.span()))?,
        }
    }

    *lex = string_lexer.morph();

    Ok(res)
}
