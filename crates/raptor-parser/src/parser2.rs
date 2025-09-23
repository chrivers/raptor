use std::sync::Arc;

use logos::Lexer;

use crate::{
    ParseError, ParseResult,
    ast::{InstRun, Instruction, Origin, Statement},
    lexer::WordToken,
};

pub struct Parser<'src> {
    lexer: Lexer<'src, WordToken<'src>>,
}

trait Required<T> {
    fn required(self) -> ParseResult<T>;
}

impl<'src> Required<WordToken<'src>> for Option<ParseResult<WordToken<'src>>> {
    fn required(self) -> ParseResult<WordToken<'src>> {
        match self {
            Some(inner) => Ok(inner?),
            None => Err(ParseError::UnexpectedEof),
        }
    }
}

trait Lex<'a, T> {
    fn bareword(&self) -> ParseResult<&'a str>;
}

impl<'src> Lex<'src, Self> for WordToken<'src> {
    fn bareword(&self) -> ParseResult<&'src str> {
        if let Self::Bareword(word) = self {
            Ok(word)
        } else {
            Err(ParseError::ExpectedWord)
        }
    }
}

impl<'src> Parser<'src> {
    #[must_use]
    pub const fn new(lexer: Lexer<'src, WordToken<'src>>) -> Self {
        Self { lexer }
    }

    fn word(&mut self) -> Option<ParseResult<WordToken<'src>>> {
        self.lexer.next().map(|word| word.map_err(ParseError::from))
    }

    #[allow(clippy::needless_continue)]
    pub fn parse_run(&mut self) -> ParseResult<InstRun> {
        let mut run = vec![];

        loop {
            let token = self.word().required()?;
            match token {
                WordToken::Bareword(word) => run.push(word.to_string()),
                WordToken::Newline(_) | WordToken::Comment(_) => break,
                WordToken::String(word) => run.push(word),
                WordToken::Whitespace(_) => continue,
            }
        }

        Ok(InstRun { run })
    }

    pub fn statement(&mut self) -> ParseResult<Option<Statement>> {
        let word = self.word();
        if word.is_none() {
            return Ok(None);
        }

        let origin = Origin::new(Arc::new("foo".into()), 0..0);
        let inst = match word.required()?.bareword()? {
            "RUN" => Instruction::Run(self.parse_run()?),

            _ => return Err(ParseError::ExpectedWord),
        };

        Ok(Some(Statement { inst, origin }))
    }

    pub fn file(&mut self) -> ParseResult<Vec<Statement>> {
        let mut res = vec![];

        while let Some(stmt) = self.statement()? {
            res.push(stmt);
        }

        Ok(res)
    }
}
