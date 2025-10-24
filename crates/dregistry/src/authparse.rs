use std::collections::HashMap;

use logos::{Lexer, Logos, Span};

use crate::error::DResult;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Default)]
pub enum LexerError {
    #[default]
    #[error("Lexer error")]
    LexerError,

    #[error("Unterminated string literal at position {}", .0.start)]
    UnterminatedString(Span),

    #[error("Unsupported string escape: {0:?}")]
    BadEscape(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    LexerError(#[from] LexerError),

    #[error("Expected {0}")]
    Expected(&'static str),

    #[error("Expected {} but found {}", .exp.description(), .found.description())]
    Mismatch { exp: Token, found: Token },
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(error = LexerError)]
pub enum Token {
    #[regex("[a-zA-Z0-9!#$%&'*+\\-.^_`|~]+")]
    Token,

    #[token("=")]
    Equals,

    #[token(",", logos::skip)]
    Comma,

    #[regex("[ ]+", logos::skip)]
    Space,

    #[token("\"", string_callback)]
    String(String),

    Eof,
}

impl Token {
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Equals => "'=' (equals)",
            Self::Comma => "',' (comma)",
            Self::String(_) => "<string>",
            Self::Eof => "<end of file>",
            Self::Token => "<token>",
            Self::Space => "<space>",
        }
    }
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

pub struct Parser<'src> {
    lexer: Lexer<'src, Token>,
}

type ParseResult<T> = Result<T, ParseError>;

impl<'src> Parser<'src> {
    fn new(input: &'src str) -> Self {
        Self {
            lexer: Token::lexer(input),
        }
    }

    fn next(&mut self) -> ParseResult<Token> {
        self.lexer
            .next()
            .unwrap_or(Ok(Token::Eof))
            .map_err(ParseError::from)
    }

    fn expect(&mut self, exp: &Token) -> ParseResult<()> {
        let next = self.next()?;
        if &next == exp {
            Ok(())
        } else {
            Err(ParseError::Mismatch {
                exp: exp.clone(),
                found: next,
            })
        }
    }

    fn accept(&mut self, exp: &Token) -> ParseResult<bool> {
        if &self.peek()? == exp {
            self.next()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn peek(&self) -> ParseResult<Token> {
        // FIXME: do away with the .clone() here
        self.lexer
            .clone()
            .next()
            .unwrap_or(Ok(Token::Eof))
            .map_err(ParseError::from)
    }

    fn token_string(&self) -> String {
        self.lexer.slice().to_string()
    }

    fn parse_token_or_quoted(&mut self) -> ParseResult<String> {
        match self.next()? {
            Token::Token => Ok(self.token_string()),
            Token::String(value) => Ok(value),
            _ => Err(ParseError::Expected("token or quoted-string")),
        }
    }

    fn parse_auth_param(&mut self) -> ParseResult<Option<(String, String)>> {
        if self.peek()? == Token::Eof {
            return Ok(None);
        }
        let state = self.lexer.clone();
        self.expect(&Token::Token)?;
        let key = self.token_string();
        if !self.accept(&Token::Equals)? {
            self.lexer = state;
            return Ok(None);
        }
        let value = self.parse_token_or_quoted()?;
        Ok(Some((key, value)))
    }

    fn parse_challenge(&mut self) -> ParseResult<(String, HashMap<String, String>)> {
        self.expect(&Token::Token)?;
        let name = self.lexer.slice().to_string();

        let mut map = HashMap::new();
        while let Some((key, value)) = self.parse_auth_param()? {
            map.insert(key, value);
        }
        Ok((name, map))
    }

    pub fn parse_auth(mut self) -> ParseResult<HashMap<String, HashMap<String, String>>> {
        let mut methods = HashMap::new();

        loop {
            let (name, map) = self.parse_challenge()?;
            methods.insert(name, map);
            if self.peek()? != Token::Token {
                break;
            }
        }

        Ok(methods)
    }
}

pub fn parse_www_authenticate(input: &str) -> DResult<HashMap<String, HashMap<String, String>>> {
    Ok(Parser::new(input).parse_auth()?)
}

#[cfg(test)]
mod tests {
    use maplit::hashmap;
    use pretty_assertions::assert_eq;

    use crate::authparse::parse_www_authenticate;

    macro_rules! parse_test {
        ($input:expr, $exp:expr) => {
            let res = parse_www_authenticate($input).unwrap();

            assert_eq!(res, $exp);
        };
    }

    #[test]
    fn test_parse_rfc() {
        let input =
            r#"Newauth realm="apps", type=1, title="Login to \"apps\"", Basic realm="simple""#;

        parse_test!(
            &input,
            hashmap! {
                "Newauth".into() => hashmap! {
                    "realm".into() => "apps".into(),
                    "type".into() => "1".into(),
                    "title".into() => "Login to \"apps\"".into(),
                },
                "Basic".into() => hashmap! {
                    "realm".into() => "simple".into(),
                },
            }
        );
    }

    #[test]
    fn test_parse_gitlab() {
        let input = r#"Bearer realm="https://gitlab.com/jwt/auth",service="container_registry",scope="repository:gitlab-org/public-image-archive/gitlab-ce:pull""#;

        parse_test!(
            input,
            hashmap! {
                "Bearer".into() => hashmap! {
                    "realm".into() => "https://gitlab.com/jwt/auth".into(),
                    "service".into() => "container_registry".into(),
                    "scope".into() => "repository:gitlab-org/public-image-archive/gitlab-ce:pull".into(),
                }
            }
        );
    }

    #[test]
    fn test_parse_docker() {
        let input = r#"Bearer realm="https://auth.docker.io/token",service="registry.docker.io""#;

        parse_test!(
            input,
            hashmap! {
                "Bearer".into() => hashmap! {
                    "realm".into() => "https://auth.docker.io/token".into(),
                    "service".into() => "registry.docker.io".into(),
                }
            }
        );
    }
}
