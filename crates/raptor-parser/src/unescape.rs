use lalrpop_util::ParseError as LalrParseError;
use lalrpop_util::lexer::Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParseState {
    Char,
    Escape,
}

pub fn unescape(val: &str) -> Result<String, LalrParseError<usize, Token, &'static str>> {
    let mut res = String::new();
    let mut state = ParseState::Char;

    for c in val.chars() {
        match state {
            ParseState::Char => {
                if c == '\\' {
                    state = ParseState::Escape;
                } else {
                    res.push(c);
                }
            }
            ParseState::Escape => {
                match c {
                    'n' => res.push('\n'),
                    't' => res.push('\t'),
                    '\\' => res.push('\\'),
                    _ => {
                        return Err(LalrParseError::User {
                            error: "invalid string escape",
                        });
                    }
                }
                state = ParseState::Char;
            }
        }
    }

    if state != ParseState::Char {
        return Err(LalrParseError::User {
            error: "invalid string escape",
        });
    }

    Ok(res)
}
