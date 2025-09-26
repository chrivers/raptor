use std::sync::Arc;

use camino::Utf8PathBuf;
use clap::Parser as _;
use logos::{Lexer, Logos};

use crate::ast::{
    Chown, FromSource, InstCmd, InstCopy, InstEntrypoint, InstEnv, InstEnvAssign, InstFrom,
    InstInvoke, InstMkdir, InstMount, InstRun, InstWorkdir, InstWrite, Instruction, MountOptions,
    MountType, Origin, Statement,
};
use crate::lexer::WordToken;
use crate::util::module_name::ModuleName;
use crate::{ParseError, ParseResult};

pub struct Parser<'src> {
    lexer: Lexer<'src, WordToken<'src>>,
    filename: Arc<Utf8PathBuf>,
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
struct FileOpts {
    #[arg(long, value_parser = parse_chmod_permission)]
    chmod: Option<u32>,

    #[arg(long, value_parser = parse_chown)]
    chown: Option<Chown>,
}

#[derive(clap::Args, Debug)]
#[group(multiple = false)]
struct MountTypeArg {
    #[arg(long)]
    simple: bool,

    #[arg(long)]
    layers: bool,

    #[arg(long)]
    overlay: bool,
}

impl MountTypeArg {
    pub const fn mtype(&self) -> Option<MountType> {
        match (self.simple, self.layers, self.overlay) {
            (false, false, false) => None,
            (true, _, _) => Some(MountType::Simple),
            (_, true, _) => Some(MountType::Layers),
            (_, _, true) => Some(MountType::Overlay),
        }
    }
}

#[derive(clap::Args, Debug)]
struct MountOpts {
    #[clap(flatten)]
    mtype: MountTypeArg,
}

#[derive(clap::Parser, Debug)]
#[clap(disable_help_flag = true)]
#[command(name = "COPY")]
struct CopyArgs {
    #[clap(flatten)]
    opts: FileOpts,

    #[arg(num_args = 2.., value_names = ["source", "dest"])]
    files: Vec<Utf8PathBuf>,
}

#[derive(clap::Parser, Debug)]
#[clap(disable_help_flag = true)]
#[command(name = "WRITE")]
struct WriteArgs {
    #[clap(flatten)]
    opts: FileOpts,

    body: String,

    dest: Utf8PathBuf,
}

#[derive(clap::Parser, Debug)]
#[clap(disable_help_flag = true)]
#[command(name = "MKDIR")]
struct MkdirArgs {
    #[clap(flatten)]
    opts: FileOpts,

    #[arg(short = 'p', default_value_t = false)]
    parents: bool,

    dest: Utf8PathBuf,
}

#[derive(clap::Parser, Debug)]
#[clap(disable_help_flag = true)]
#[command(name = "MOUNT")]
struct MountArgs {
    #[clap(flatten)]
    opts: MountOpts,

    name: String,

    dest: Utf8PathBuf,
}

trait Lex<'a, 'b, T> {
    fn bareword(&self) -> ParseResult<&'a str>;
    fn value(&'b self) -> ParseResult<&'b str>;
    fn path(&self) -> ParseResult<Utf8PathBuf>;
}

impl<'src, 'this> Lex<'src, 'this, Self> for WordToken<'src> {
    fn bareword(&self) -> ParseResult<&'src str> {
        if let Self::Bareword(word) = self {
            Ok(word)
        } else {
            Err(ParseError::ExpectedWord)
        }
    }

    #[allow(clippy::match_same_arms)]
    fn value(&'this self) -> ParseResult<&'this str> {
        match self {
            WordToken::Bareword(word) => Ok(word),
            WordToken::String(string) => Ok(string.as_ref()),
            WordToken::Newline(_) => Err(ParseError::ExpectedWord),
            WordToken::Comment(_) => Err(ParseError::ExpectedWord),
            WordToken::Whitespace(_) => Err(ParseError::ExpectedWord),
            WordToken::Eof => Err(ParseError::UnexpectedEof),
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
    pub const fn new(lexer: Lexer<'src, WordToken<'src>>, filename: Arc<Utf8PathBuf>) -> Self {
        Self { lexer, filename }
    }

    fn next(&mut self) -> ParseResult<WordToken<'src>> {
        self.lexer
            .next()
            .unwrap_or(Ok(WordToken::Eof))
            .map_err(ParseError::from)
    }

    fn peek(&self) -> ParseResult<WordToken<'src>> {
        // FIXME: do away with the .clone() here
        self.lexer
            .clone()
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

    pub fn parse_invoke(&mut self) -> ParseResult<InstInvoke> {
        let args = self.consume_line()?;

        Ok(InstInvoke { args })
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

    pub fn parse_env_assign(&mut self) -> ParseResult<Option<InstEnvAssign>> {
        if let WordToken::Newline(_) = self.peek()? {
            return Ok(None);
        }

        let word = self.word()?;
        let value = word.value()?;

        let assign = if let Some((head, tail)) = value.split_once('=') {
            InstEnvAssign {
                key: head.to_string(),
                value: tail.to_string(),
            }
        } else {
            InstEnvAssign {
                key: value.to_string(),
                value: value.to_string(),
            }
        };

        Ok(Some(assign))
    }

    pub fn parse_env(&mut self) -> ParseResult<InstEnv> {
        let mut env = vec![];
        while let Some(assign) = self.parse_env_assign()? {
            env.push(assign);
        }
        self.end_of_line()?;

        Ok(InstEnv { env })
    }

    pub fn parse_write(&mut self) -> ParseResult<InstWrite> {
        // clap requires dummy string to simulate argv[0]
        let mut copy = vec![String::new()];
        self.consume_line_to(&mut copy)?;

        let WriteArgs {
            opts: FileOpts { chmod, chown },
            body,
            dest,
        } = WriteArgs::try_parse_from(copy)?;

        Ok(InstWrite {
            dest,
            body,
            chmod,
            chown,
        })
    }

    pub fn parse_mkdir(&mut self) -> ParseResult<InstMkdir> {
        // clap requires dummy string to simulate argv[0]
        let mut copy = vec![String::new()];
        self.consume_line_to(&mut copy)?;

        let MkdirArgs {
            opts: FileOpts { chmod, chown },
            parents,
            dest,
        } = MkdirArgs::try_parse_from(copy)?;

        Ok(InstMkdir {
            dest,
            chmod,
            chown,
            parents,
        })
    }

    #[allow(clippy::option_if_let_else)]
    pub fn parse_from(&mut self) -> ParseResult<InstFrom> {
        let word = self.word()?.bareword()?;

        let from = if let Some(docker) = word.strip_prefix("docker://") {
            FromSource::Docker(docker.to_string())
        } else {
            FromSource::Raptor(ModuleName::new(
                word.split('.').map(str::to_string).collect(),
            ))
        };

        self.end_of_line()?;

        Ok(InstFrom { from })
    }

    #[allow(clippy::option_if_let_else)]
    pub fn parse_mount(&mut self) -> ParseResult<InstMount> {
        // clap requires dummy string to simulate argv[0]
        let mut copy = vec![String::new()];
        self.consume_line_to(&mut copy)?;

        let MountArgs { opts, name, dest } = MountArgs::try_parse_from(copy)?;

        let mtype = opts.mtype.mtype().unwrap_or(MountType::Simple);

        Ok(InstMount {
            opts: MountOptions { mtype },
            name,
            dest,
        })
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

        if matches!(word, WordToken::Newline(_)) {
            return self.statement();
        }

        let start = self.lexer.span().start;

        let inst = match word.bareword()? {
            "FROM" => Instruction::From(self.parse_from()?),
            "MOUNT" => Instruction::Mount(self.parse_mount()?),
            /* RENDER */
            "WRITE" => Instruction::Write(self.parse_write()?),
            "MKDIR" => Instruction::Mkdir(self.parse_mkdir()?),
            "COPY" => Instruction::Copy(self.parse_copy()?),
            /* INCLUDE */
            "INVOKE" => Instruction::Invoke(self.parse_invoke()?),
            "RUN" => Instruction::Run(self.parse_run()?),
            "ENV" => Instruction::Env(self.parse_env()?),
            "WORKDIR" => Instruction::Workdir(self.parse_workdir()?),
            "ENTRYPOINT" => Instruction::Entrypoint(self.parse_entrypoint()?),
            "CMD" => Instruction::Cmd(self.parse_cmd()?),
            _ => return Err(ParseError::ExpectedWord),
        };

        let end = self.lexer.span().end;

        let origin = Origin::new(self.filename.clone(), start..end - 1);

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

pub fn parse(name: &str, buf: &str) -> ParseResult<Vec<Statement>> {
    let lexer = WordToken::lexer(buf);
    let mut parser = Parser::new(lexer, Arc::new(name.into()));

    parser.file()
}
