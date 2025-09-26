use std::sync::Arc;

use camino::Utf8PathBuf;
use clap::Parser as _;
use logos::Lexer;

use crate::ast::{
    Chown, InstCmd, InstCopy, InstEntrypoint, InstRun, InstWorkdir, Instruction, Origin, Statement,
};
use crate::lexer::WordToken;
use crate::{ParseError, ParseResult};

pub struct Parser<'src> {
    lexer: Lexer<'src, WordToken<'src>>,
}

fn parse_chmod_permission(string: &str) -> Result<u32, ParseError> {
    if !(3..=4).contains(&string.len()) {
        return Err(ParseError::InvalidPermissionMask);
    }
    Ok(u32::from_str_radix(string, 8)?)
}

fn parse_chown(string: &str) -> Result<Chown, ParseError> {
    let res = if let Some((head, tail)) = string.split_once(':') {
        match (head, tail) {
            ("", "") => return Err(ParseError::ExpectedWord),
            (head, "") => Chown {
                user: Some(head.to_string()),
                group: Some(head.to_string()),
            },
            ("", tail) => Chown {
                user: None,
                group: Some(tail.to_string()),
            },
            (head, tail) => Chown {
                user: Some(head.to_string()),
                group: Some(tail.to_string()),
            },
        }
    } else {
        Chown {
            user: Some(string.to_string()),
            group: None,
        }
    };

    Ok(res)
}

#[derive(clap::Args, Debug)]
#[command(about = "bar", name = "COPY", long_about = "foo")]
struct FileOpts {
    #[arg(long, value_parser = parse_chmod_permission)]
    chmod: Option<u32>,

    #[arg(long, value_parser = parse_chown)]
    chown: Option<Chown>,
}

#[derive(clap::Parser, Debug)]
#[clap(disable_help_flag = true)]
#[command(about = "bar", name = "COPY", long_about = "foo")]
struct CopyArgs {
    #[clap(flatten)]
    opts: FileOpts,

    #[arg(num_args = 2.., value_names = ["source", "dest"])]
    files: Vec<Utf8PathBuf>,
}

trait Lex<'a, T> {
    fn bareword(&self) -> ParseResult<&'a str>;
    fn path(&self) -> ParseResult<Utf8PathBuf>;
}

impl<'src> Lex<'src, Self> for WordToken<'src> {
    fn bareword(&self) -> ParseResult<&'src str> {
        if let Self::Bareword(word) = self {
            Ok(word)
        } else {
            Err(ParseError::ExpectedWord)
        }
    }

    #[allow(clippy::match_same_arms)]
    fn path(&self) -> ParseResult<Utf8PathBuf> {
        match self {
            WordToken::Bareword(word) => Ok(word.into()),
            WordToken::String(string) => Ok(string.into()),
            WordToken::Newline(_) => Err(ParseError::ExpectedWord),
            WordToken::Comment(_) => Err(ParseError::ExpectedWord),
            WordToken::Whitespace(_) => Err(ParseError::ExpectedWord),
            WordToken::Eof => Err(ParseError::UnexpectedEof),
        }
    }
}

impl<'src> Parser<'src> {
    #[must_use]
    pub const fn new(lexer: Lexer<'src, WordToken<'src>>) -> Self {
        Self { lexer }
    }

    fn next(&mut self) -> ParseResult<WordToken<'src>> {
        self.lexer
            .next()
            .unwrap_or(Ok(WordToken::Eof))
            .map_err(ParseError::from)
    }

    fn word(&mut self) -> ParseResult<WordToken<'src>> {
        loop {
            let next = self.next()?;
            if !matches!(next, WordToken::Whitespace(_)) {
                return Ok(next);
            }
        }
    }

    fn end_of_line(&mut self) -> ParseResult<()> {
        loop {
            match self.next()? {
                WordToken::Newline(_) | WordToken::Comment(_) => break,
                WordToken::Whitespace(_) => {}
                WordToken::Bareword(_) | WordToken::String(_) => {
                    return Err(ParseError::ExpectedEol);
                }
                WordToken::Eof => return Err(ParseError::UnexpectedEof),
            }
        }

        Ok(())
    }

    #[allow(clippy::needless_continue)]
    fn consume_line_to(&mut self, args: &mut Vec<String>) -> ParseResult<()> {
        loop {
            let token = self.word()?;
            match token {
                WordToken::Bareword(word) => args.push(word.to_string()),
                WordToken::String(word) => args.push(word),
                WordToken::Whitespace(_) => continue,
                WordToken::Newline(_) | WordToken::Comment(_) | WordToken::Eof => break,
            }
        }

        Ok(())
    }

    fn consume_line(&mut self) -> ParseResult<Vec<String>> {
        let mut args = vec![];
        self.consume_line_to(&mut args)?;
        Ok(args)
    }

    pub fn parse_run(&mut self) -> ParseResult<InstRun> {
        let run = self.consume_line()?;

        Ok(InstRun { run })
    }

    pub fn parse_entrypoint(&mut self) -> ParseResult<InstEntrypoint> {
        let entrypoint = self.consume_line()?;

        Ok(InstEntrypoint { entrypoint })
    }

    pub fn parse_cmd(&mut self) -> ParseResult<InstCmd> {
        let cmd = self.consume_line()?;

        Ok(InstCmd { cmd })
    }

    pub fn parse_workdir(&mut self) -> ParseResult<InstWorkdir> {
        let dir = self.word()?.path()?;
        self.end_of_line()?;

        Ok(InstWorkdir { dir })
    }

    pub fn parse_copy(&mut self) -> ParseResult<InstCopy> {
        // clap requires dummy string to simulate argv[0]
        let mut copy = vec![String::new()];
        self.consume_line_to(&mut copy)?;

        let CopyArgs {
            opts: FileOpts { chmod, chown },
            mut files,
        } = CopyArgs::try_parse_from(copy)?;

        // clap does not support variable arguments before fixed argument,
        // but we know the destination is the last name.
        //
        // Safety: clap requires at least 2 arguments.
        let dest = files.pop().unwrap();

        Ok(InstCopy {
            dest,
            srcs: files,
            chmod,
            chown,
        })
    }

    pub fn statement(&mut self) -> ParseResult<Option<Statement>> {
        let word = self.word()?;

        if matches!(word, WordToken::Eof) {
            return Ok(None);
        }

        let origin = Origin::new(Arc::new("foo".into()), 0..0);

        let inst = match word.bareword()? {
            /* FROM */
            /* MOUNT */
            /* RENDER */
            /* WRITE */
            /* MKDIR */
            "COPY" => Instruction::Copy(self.parse_copy()?),
            /* INCLUDE */
            /* INVOKE */
            "RUN" => Instruction::Run(self.parse_run()?),
            /* ENV */
            "WORKDIR" => Instruction::Workdir(self.parse_workdir()?),
            "ENTRYPOINT" => Instruction::Entrypoint(self.parse_entrypoint()?),
            "CMD" => Instruction::Cmd(self.parse_cmd()?),
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
