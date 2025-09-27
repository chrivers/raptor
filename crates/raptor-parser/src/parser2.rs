use std::sync::Arc;

use camino::Utf8PathBuf;
use clap::Parser as _;
use logos::{Lexer, Logos};
use minijinja::Value;

use crate::ast::{
    Chown, Expression, FromSource, IncludeArg, InstCmd, InstCopy, InstEntrypoint, InstEnv,
    InstEnvAssign, InstFrom, InstInclude, InstInvoke, InstMkdir, InstMount, InstRender, InstRun,
    InstWorkdir, InstWrite, Instruction, Lookup, MountOptions, MountType, Origin, Statement,
};
use crate::lexer::Token;
use crate::util::module_name::ModuleName;
use crate::{ParseError, ParseResult};

pub struct Parser<'src> {
    lexer: Lexer<'src, Token>,
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

#[derive(clap::Parser, Debug)]
#[clap(disable_help_flag = true)]
#[command(name = "RENDER")]
struct RenderArgs {
    #[clap(flatten)]
    opts: FileOpts,

    src: Utf8PathBuf,

    dest: Utf8PathBuf,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    remainder: Vec<String>,
}

impl<'src> Parser<'src> {
    #[must_use]
    pub const fn new(lexer: Lexer<'src, Token>, filename: Arc<Utf8PathBuf>) -> Self {
        Self { lexer, filename }
    }

    fn next(&mut self) -> ParseResult<Token> {
        self.lexer
            .next()
            .unwrap_or(Ok(Token::Eof))
            .map_err(ParseError::from)
    }

    fn expect(&mut self, exp: &Token) -> ParseResult<Token> {
        let next = self.next()?;
        if &next == exp {
            Ok(next)
        } else {
            Err(ParseError::ExpectedWord)
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

    fn bareword(&mut self) -> ParseResult<&'src str> {
        if self.word()? == Token::Bareword {
            Ok(self.lexer.slice())
        } else {
            Err(ParseError::ExpectedWord)
        }
    }

    #[allow(clippy::match_same_arms)]
    fn value(&mut self) -> ParseResult<String> {
        match self.next()? {
            Token::Bareword => Ok(self.lexer.slice().to_string()),
            Token::String(string) => Ok(string),
            Token::Eof => Err(ParseError::UnexpectedEof),
            Token::Newline => Err(ParseError::ExpectedWord),
            Token::Comment => Err(ParseError::ExpectedWord),
            _ => Err(ParseError::ExpectedWord),
        }
    }

    fn trim(&mut self) -> ParseResult<()> {
        while self.peek()?.is_whitespace() {
            self.next()?;
        }

        Ok(())
    }

    #[allow(clippy::match_same_arms)]
    fn path(&mut self) -> ParseResult<Utf8PathBuf> {
        let mut res = String::new();
        loop {
            let state = self.lexer.clone();
            match self.next()? {
                Token::Bareword => res.push_str(self.lexer.slice()),
                Token::String(string) => res.push_str(&string),
                Token::Colon | Token::Dot | Token::Slash => {
                    res.push_str(self.lexer.slice());
                }
                _ => {
                    self.lexer = state;
                    break;
                }
            }
        }

        if res.is_empty() {
            return Err(ParseError::ExpectedWord);
        }

        Ok(res.into())
    }

    fn module_name(&mut self) -> ParseResult<ModuleName> {
        let word = self.bareword()?;
        let res = ModuleName::new(word.split('.').map(str::to_string).collect());

        Ok(res)
    }

    fn word(&mut self) -> ParseResult<Token> {
        loop {
            let next = self.next()?;
            if !matches!(next, Token::Whitespace) {
                return Ok(next);
            }
        }
    }

    fn end_of_line(&mut self) -> ParseResult<()> {
        loop {
            match self.next()? {
                Token::Newline | Token::Comment => break,
                Token::Whitespace => {}
                Token::Eof => return Err(ParseError::UnexpectedEof),
                _ => return Err(ParseError::ExpectedEol),
            }
        }

        Ok(())
    }

    #[allow(clippy::needless_continue)]
    fn consume_line_to(&mut self, args: &mut Vec<String>) -> ParseResult<()> {
        let mut value = String::new();

        loop {
            let token = self.next()?;
            match token {
                Token::String(word) => args.push(word),
                Token::Whitespace => {
                    if !value.is_empty() {
                        let mut val = String::new();
                        std::mem::swap(&mut value, &mut val);
                        args.push(val);
                    }
                }
                Token::Newline | Token::Comment | Token::Eof => break,
                _ => value.push_str(self.lexer.slice()),
            }
        }

        if !value.is_empty() {
            args.push(value);
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
        self.trim()?;
        let dir = self.path()?;
        self.end_of_line()?;

        Ok(InstWorkdir { dir })
    }

    pub fn parse_env_assign(&mut self) -> ParseResult<Option<InstEnvAssign>> {
        if self.peek()? == Token::Newline {
            return Ok(None);
        }

        let ident = self.value()?;

        let assign = if self.peek()? == Token::Equals {
            self.next()?;
            let value = self.bareword()?;
            InstEnvAssign {
                key: ident,
                value: value.to_string(),
            }
        } else {
            InstEnvAssign {
                key: ident.clone(),
                value: ident,
            }
        };

        Ok(Some(assign))
    }

    pub fn parse_env(&mut self) -> ParseResult<InstEnv> {
        let mut env = vec![];
        while let Some(assign) = self.parse_env_assign()? {
            env.push(assign);
            self.trim()?;
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
        let state = self.lexer.clone();
        let next = self.bareword()?;
        self.lexer = state;

        let from = if next == "docker" {
            self.expect(&Token::Bareword)?;
            self.expect(&Token::Colon)?;
            self.expect(&Token::Slash)?;
            self.expect(&Token::Slash)?;
            let mut docker = String::new();
            loop {
                let state = self.lexer.clone();
                let next = self.next()?;
                match next {
                    Token::LBracket
                    | Token::RBracket
                    | Token::LBrace
                    | Token::RBrace
                    | Token::Colon
                    | Token::Equals
                    | Token::Comma
                    | Token::Slash
                    | Token::Dot
                    | Token::Bareword => {
                        docker.push_str(self.lexer.slice());
                    }
                    Token::Newline
                    | Token::Comment
                    | Token::String(_)
                    | Token::Whitespace
                    | Token::Eof => {
                        self.lexer = state;
                        break;
                    }
                }
            }
            FromSource::Docker(docker.to_string())
        } else {
            FromSource::Raptor(self.module_name()?)
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

    pub fn parse_include_arg(&mut self) -> ParseResult<Option<IncludeArg>> {
        if self.peek()? == Token::Newline {
            return Ok(None);
        }

        let value = self.value()?;

        let Some((head, tail)) = value.split_once('=') else {
            return Err(ParseError::ExpectedWord);
        };

        let name = head.to_string();

        let value = if tail.contains('.') {
            let path = ModuleName::new(tail.split('.').map(str::to_string).collect());
            Expression::Lookup(Lookup::new(path, Origin::new(self.filename.clone(), 0..0)))
        } else {
            Expression::Value(Value::from_serialize(tail))
        };

        let arg = IncludeArg { name, value };

        Ok(Some(arg))
    }

    pub fn parse_include(&mut self) -> ParseResult<InstInclude> {
        let src = self.module_name()?;

        let mut args = vec![];
        while let Some(arg) = self.parse_include_arg()? {
            args.push(arg);
        }
        self.end_of_line()?;

        Ok(InstInclude { src, args })
    }

    pub fn parse_render(&mut self) -> ParseResult<InstRender> {
        // clap requires dummy string to simulate argv[0]
        let mut copy = vec![String::new()];
        self.consume_line_to(&mut copy)?;

        let RenderArgs {
            opts: FileOpts { chmod, chown },
            src,
            dest,
            ..
        } = RenderArgs::try_parse_from(&copy)?;

        Ok(InstRender {
            src,
            dest,
            chmod,
            chown,
            args: vec![],
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
        let start = self.lexer.span().start;

        let word = self.word()?;
        let inst = self.lexer.slice();
        self.trim()?;

        if matches!(word, Token::Eof) {
            return Ok(None);
        }

        if matches!(word, Token::Newline) {
            return self.statement();
        }

        let inst = match inst {
            "FROM" => Instruction::From(self.parse_from()?),
            "MOUNT" => Instruction::Mount(self.parse_mount()?),
            "RENDER" => Instruction::Render(self.parse_render()?),
            "WRITE" => Instruction::Write(self.parse_write()?),
            "MKDIR" => Instruction::Mkdir(self.parse_mkdir()?),
            "COPY" => Instruction::Copy(self.parse_copy()?),
            "INCLUDE" => Instruction::Include(self.parse_include()?),
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
    let lexer = Token::lexer(buf);
    let mut parser = Parser::new(lexer, Arc::new(name.into()));

    parser.file()
}
