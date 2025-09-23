use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
pub enum WordToken<'a> {
    #[regex("[^ \n\t\"]+")]
    Text(&'a str),

    #[token("\n")]
    Newline(&'a str),

    #[regex(r"#.+\n")]
    Comment(&'a str),

    #[token("\"", string_callback)]
    String(String),

    #[regex(r"( |\t|\\\n)+", priority = 3)]
    Whitespace(&'a str),
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
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

fn string_callback<'a>(lex: &mut Lexer<'a, WordToken<'a>>) -> Result<String, ()> {
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
            StringToken::Newline => return Err(()),
        }
    }

    *lex = string_lexer.morph();

    Ok(res)
}
