use std::collections::BTreeMap;
use std::sync::Arc;

use camino::Utf8PathBuf;
use logos::{Lexer, Logos};
use minijinja::Value;

use crate::ast::{
    Chown, Expression, FromSource, IncludeArg, InstCmd, InstCopy, InstEntrypoint, InstEnv,
    InstEnvAssign, InstFrom, InstInclude, InstInvoke, InstMkdir, InstMount, InstRender, InstRun,
    InstWorkdir, InstWrite, Instruction, Lookup, MountOptions, MountType, Origin, Statement,
};
use crate::lexer::Token;
use crate::util::Location;
use crate::util::module_name::ModuleName;
use crate::{ParseError, ParseResult};

pub struct Parser<'src> {
    lexer: Lexer<'src, Token>,
    filename: Arc<Utf8PathBuf>,
}

fn parse_chmod_permission(string: &str) -> ParseResult<u32> {
    if !(3..=4).contains(&string.len()) {
        return Err(ParseError::InvalidPermissionMask);
    }
    Ok(u32::from_str_radix(string, 8)?)
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

    fn accept(&mut self, exp: &Token) -> ParseResult<bool> {
        if &self.peek()? == exp {
            self.next()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expect(&mut self, exp: &Token) -> ParseResult<()> {
        let next = self.next()?;
        if &next == exp {
            Ok(())
        } else {
            Err(ParseError::Expected(exp.name()))
        }
    }

    fn expect_trimmed(&mut self, exp: &Token) -> ParseResult<()> {
        self.trim()?;
        self.expect(exp)?;
        self.trim()?;

        Ok(())
    }

    fn accept_trimmed(&mut self, exp: &Token) -> ParseResult<bool> {
        self.trim()?;
        let accepted = self.accept(exp)?;
        if accepted {
            self.trim()?;
        }

        Ok(accepted)
    }

    fn peek(&self) -> ParseResult<Token> {
        // FIXME: do away with the .clone() here
        self.lexer
            .clone()
            .next()
            .unwrap_or(Ok(Token::Eof))
            .map_err(ParseError::from)
    }

    fn token(&self) -> &'src str {
        self.lexer.slice()
    }

    fn token_string(&self) -> String {
        self.lexer.slice().to_string()
    }

    fn bareword(&mut self) -> ParseResult<&'src str> {
        let word = self.next()?;
        if word == Token::Bareword {
            Ok(self.token())
        } else {
            Err(ParseError::Mismatch {
                exp: Token::Bareword,
                found: word,
            })
        }
    }

    fn value(&mut self) -> ParseResult<String> {
        let res = match self.next()? {
            Token::Bareword => Ok(self.token_string()),
            Token::String(string) => Ok(string),
            _ => Err(ParseError::Expected("value")),
        };

        self.trim()?;

        res
    }

    fn trim(&mut self) -> ParseResult<()> {
        while self.peek()? == Token::Whitespace {
            self.next()?;
        }

        Ok(())
    }

    fn parse_path(&mut self) -> ParseResult<Utf8PathBuf> {
        let mut res = String::new();
        loop {
            let state = self.lexer.clone();
            match self.next()? {
                Token::Bareword => res.push_str(self.token()),
                Token::String(string) => res.push_str(&string),
                Token::Comment | Token::Whitespace | Token::Newline | Token::Eof => {
                    self.lexer = state;
                    break;
                }
                _ => {
                    res.push_str(self.token());
                }
            }
        }

        if res.is_empty() {
            return Err(ParseError::Expected("path"));
        }

        self.trim()?;

        Ok(res.into())
    }

    fn module_name(&mut self) -> ParseResult<ModuleName> {
        let mut words = vec![self.bareword()?.to_string()];

        while self.accept(&Token::Dot)? {
            words.push(self.bareword()?.to_string());
        }

        Ok(ModuleName::new(words))
    }

    fn end_of_line(&mut self) -> ParseResult<()> {
        loop {
            match self.next()? {
                Token::Newline | Token::Comment => break,
                Token::Whitespace => {}
                found => {
                    return Err(ParseError::Mismatch {
                        exp: Token::Newline,
                        found,
                    });
                }
            }
        }

        Ok(())
    }

    fn consume_line(&mut self) -> ParseResult<Vec<String>> {
        self.trim()?;

        let mut args = vec![];
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
                _ => value.push_str(self.token()),
            }
        }

        if !value.is_empty() {
            args.push(value);
        }

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

        let dir = self.parse_path()?;

        self.end_of_line()?;

        Ok(InstWorkdir { dir })
    }

    pub fn parse_env_assign(&mut self) -> ParseResult<Option<InstEnvAssign>> {
        if self.peek()? == Token::Newline {
            return Ok(None);
        }

        let ident = self.bareword()?.to_string();

        let assign = if self.accept(&Token::Equals)? {
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

        self.trim()?;

        Ok(Some(assign))
    }

    pub fn parse_env(&mut self) -> ParseResult<InstEnv> {
        self.trim()?;

        let mut env = vec![];
        while let Some(assign) = self.parse_env_assign()? {
            env.push(assign);
        }
        self.end_of_line()?;

        Ok(InstEnv { env })
    }

    pub fn parse_write(&mut self) -> ParseResult<InstWrite> {
        self.trim()?;

        let (chown, chmod) = self.parse_fileopts(None)?;

        let body = self.value()?;
        let dest = self.parse_path()?;

        self.end_of_line()?;

        Ok(InstWrite {
            dest,
            body,
            chmod,
            chown,
        })
    }

    pub fn parse_mkdir(&mut self) -> ParseResult<InstMkdir> {
        self.trim()?;

        let mut parents = false;
        let (chown, chmod) = self.parse_fileopts(Some(&mut parents))?;

        let dest = self.parse_path()?;

        self.end_of_line()?;

        Ok(InstMkdir {
            dest,
            chmod,
            chown,
            parents,
        })
    }

    pub fn parse_from(&mut self) -> ParseResult<InstFrom> {
        self.trim()?;

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
                    | Token::Minus
                    | Token::Number
                    | Token::Bareword => {
                        docker.push_str(self.token());
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

    pub fn parse_mount_options(&mut self) -> ParseResult<MountOptions> {
        let mut opts = MountOptions {
            mtype: MountType::Simple,
        };

        while self.accept(&Token::Minus)? {
            self.expect(&Token::Minus)?;
            self.expect(&Token::Bareword)?;

            match self.token() {
                "simple" => opts.mtype = MountType::Simple,
                "layers" => opts.mtype = MountType::Layers,
                "overlay" => opts.mtype = MountType::Overlay,
                _ => return Err(ParseError::Expected("mount option")),
            }
        }

        self.trim()?;

        Ok(opts)
    }

    pub fn parse_mount(&mut self) -> ParseResult<InstMount> {
        self.trim()?;

        let opts = self.parse_mount_options()?;

        let name = self.bareword()?.to_string();
        self.trim()?;

        let dest = self.parse_path()?;

        self.end_of_line()?;

        Ok(InstMount { opts, name, dest })
    }

    pub fn parse_list(&mut self) -> ParseResult<Value> {
        let mut list = vec![];

        self.expect_trimmed(&Token::LBracket)?;

        loop {
            if self.accept_trimmed(&Token::RBracket)? {
                break;
            }

            list.push(self.parse_value()?);

            if !self.accept_trimmed(&Token::Comma)? {
                self.expect_trimmed(&Token::RBracket)?;
                break;
            }
        }

        Ok(Value::from(list))
    }

    pub fn parse_map(&mut self) -> ParseResult<Value> {
        self.expect_trimmed(&Token::LBrace)?;

        let mut map = BTreeMap::new();
        loop {
            if self.accept_trimmed(&Token::RBrace)? {
                break;
            }

            let key = self.parse_value()?;
            self.expect_trimmed(&Token::Colon)?;
            let value = self.parse_value()?;

            map.insert(key, value);

            if !self.accept_trimmed(&Token::Comma)? {
                self.expect_trimmed(&Token::RBrace)?;
                break;
            }
        }

        Ok(Value::from(map))
    }

    pub fn parse_value(&mut self) -> ParseResult<Value> {
        self.trim()?;
        match self.peek()? {
            Token::LBracket => self.parse_list(),
            Token::LBrace => self.parse_map(),
            Token::String(value) => {
                self.next()?;
                Ok(Value::from_serialize(value))
            }
            Token::Number => {
                self.expect(&Token::Number)?;
                Ok(Value::from_serialize(self.token().parse::<i64>()?))
            }
            Token::Bareword => {
                self.expect(&Token::Bareword)?;
                let value = match self.token() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(ParseError::Expected("boolean")),
                };
                Ok(Value::from_serialize(value))
            }

            _ => Err(ParseError::Expected("value4")),
        }
    }

    pub fn parse_expression(&mut self) -> ParseResult<Expression> {
        if self.accept(&Token::Bareword)? {
            let mut path = vec![self.token_string()];

            let start = self.lexer.span().start;
            while self.accept(&Token::Dot)? {
                self.expect(&Token::Bareword)?;
                path.push(self.token_string());
            }
            let end = self.lexer.span().end;
            let origin = Origin::new(self.filename.clone(), start..end);
            Ok(Expression::Lookup(Lookup {
                path: ModuleName::new(path),
                origin,
            }))
        } else {
            Ok(Expression::Value(self.parse_value()?))
        }
    }

    pub fn parse_include_arg(&mut self) -> ParseResult<Option<IncludeArg>> {
        if self.peek()? == Token::Newline {
            return Ok(None);
        }

        let name = self.bareword()?.to_string();

        if !self.accept(&Token::Equals)? {
            let origin = Origin::new(self.filename.clone(), self.lexer.span());
            let path = ModuleName::new(vec![name.clone()]);
            let value = Expression::Lookup(Lookup { path, origin });

            return Ok(Some(IncludeArg { name, value }));
        }

        let value = self.parse_expression()?;

        let arg = IncludeArg { name, value };

        Ok(Some(arg))
    }

    pub fn parse_include(&mut self) -> ParseResult<InstInclude> {
        self.trim()?;

        let src = self.module_name()?;
        self.trim()?;

        let mut args = vec![];
        while let Some(arg) = self.parse_include_arg()? {
            args.push(arg);
            self.trim()?;
        }
        self.end_of_line()?;

        Ok(InstInclude { src, args })
    }

    pub fn parse_fileopts(
        &mut self,
        mut parent_flag: Option<&mut bool>,
    ) -> ParseResult<(Option<Chown>, Option<u32>)> {
        let mut chown = None;
        let mut chmod = None;

        while self.accept(&Token::Minus)? {
            if !self.accept(&Token::Minus)? {
                if let Some(pflag) = parent_flag.as_mut() {
                    self.bareword()?;
                    if self.token() != "p" {
                        return Err(ParseError::Expected("flag"));
                    }

                    **pflag = true;
                    self.trim()?;
                    continue;
                }
                return Err(ParseError::Expected("fileopt"));
            }

            match self.bareword()? {
                "chown" => {
                    if !self.accept(&Token::Equals)? {
                        self.expect(&Token::Whitespace)?;
                    }

                    let user = if self.peek()? == Token::Colon {
                        None
                    } else {
                        Some(self.value()?)
                    };

                    let mut group = None;

                    if self.accept(&Token::Colon)? {
                        group = if self.accept(&Token::Bareword)? {
                            Some(self.token_string())
                        } else {
                            user.clone()
                        }
                    }

                    chown = Some(Chown { user, group });
                }

                "chmod" => {
                    if !self.accept(&Token::Equals)? {
                        self.expect(&Token::Whitespace)?;
                    }

                    self.expect(&Token::Number)?;
                    chmod = Some(parse_chmod_permission(self.token())?);
                }

                _ => return Err(ParseError::Expected("file option")),
            }

            self.trim()?;
        }

        self.trim()?;

        Ok((chown, chmod))
    }

    pub fn parse_render(&mut self) -> ParseResult<InstRender> {
        self.trim()?;

        let (chown, chmod) = self.parse_fileopts(None)?;

        let src = self.parse_path()?;
        let dest = self.parse_path()?;

        let mut args = vec![];
        while let Some(arg) = self.parse_include_arg()? {
            args.push(arg);
            self.trim()?;
        }

        self.end_of_line()?;

        Ok(InstRender {
            src,
            dest,
            chmod,
            chown,
            args,
        })
    }

    pub fn parse_copy(&mut self) -> ParseResult<InstCopy> {
        self.trim()?;

        let (chown, chmod) = self.parse_fileopts(None)?;

        let mut files = vec![];
        while self.peek()? != Token::Newline {
            files.push(self.parse_path()?);
        }

        self.end_of_line()?;

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

        loop {
            match self.peek()? {
                Token::Whitespace | Token::Comment | Token::Newline => {
                    self.next()?;
                }
                Token::Eof => return Ok(None),
                _ => break,
            }
        }

        self.expect(&Token::Bareword)?;
        let inst = self.token();

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
            _ => return Err(ParseError::Expected("statement")),
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

pub fn parse(name: &str, buf: &str) -> Result<Vec<Statement>, Location<ParseError>> {
    let lexer = Token::lexer(buf);
    let path = Arc::new(Utf8PathBuf::from(name));
    let mut parser = Parser::new(lexer, path.clone());

    parser.file().map_err(|err| {
        let origin = Origin::new(path.clone(), parser.lexer.span());
        Location::make(origin, err)
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use logos::Logos;
    use minijinja::Value;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::ParseResult;
    use crate::ast::Chown;
    use crate::lexer::Token;
    use crate::parser::Parser;

    fn make_parser(input: &str) -> Parser {
        let lexer = Token::lexer(input);

        Parser::new(lexer, Arc::new("<inline>".into()))
    }

    macro_rules! list_test {
        ($src:expr, $tree:tt) => {
            let mut parser = make_parser($src);
            assert_eq!(parser.parse_list()?, Value::from_serialize(json!($tree)));
        };
    }

    macro_rules! map_test {
        ($src:expr, $tree:tt) => {
            let mut parser = make_parser($src);
            assert_eq!(parser.parse_map()?, Value::from_serialize(json!($tree)));
        };
    }

    macro_rules! fileopts_test {
        ($src:expr, $tree:tt) => {
            let mut parser = make_parser($src);

            assert_eq!(parser.parse_fileopts(None)?, $tree);
        };
    }

    macro_rules! fileopts_test_err {
        ($src:expr) => {
            let mut parser = make_parser($src);

            parser.parse_fileopts(None).unwrap_err();
        };
    }

    macro_rules! fileopts_test_chown {
        ($src:expr, $user:expr, $group:expr) => {
            let mut parser = make_parser($src);

            let mut chown = Chown::default();
            let user: Option<&str> = $user;
            let group: Option<&str> = $group;

            if let Some(user) = user {
                chown.user = Some(user.to_string());
            }
            if let Some(group) = group {
                chown.group = Some(group.to_string());
            }

            assert_eq!(parser.parse_fileopts(None)?, (Some(chown), None));
        };
    }

    macro_rules! fileopts_test_full {
        ($src:expr, $chmod:expr, $user:expr, $group:expr, $parent:expr) => {
            let mut parser = make_parser($src);

            let mut parent = false;
            let mut chown = Chown::default();
            let user: Option<&str> = $user;
            let group: Option<&str> = $group;

            if let Some(user) = user {
                chown.user = Some(user.to_string());
            }
            if let Some(group) = group {
                chown.group = Some(group.to_string());
            }

            assert_eq!(
                parser.parse_fileopts(Some(&mut parent))?,
                (
                    if user.is_some() || group.is_some() {
                        Some(chown)
                    } else {
                        None
                    },
                    $chmod
                )
            );
            assert_eq!(parent, $parent);
        };
    }

    #[test]
    fn parse_list() -> ParseResult<()> {
        list_test!("[]", []);
        list_test!("[0]", [0]);
        list_test!("[1234]", [1234]);
        list_test!("[1,2,3,4]", [1, 2, 3, 4]);
        list_test!("[ 1, 2,3 , 4 ]", [1, 2, 3, 4]);
        list_test!(" [ 1 ] ", [1]);

        Ok(())
    }

    #[test]
    fn parse_map() -> ParseResult<()> {
        map_test!("{}", {});
        map_test!(r#"{"foo":"bar"}"#, {"foo": "bar"});
        map_test!(r#" { "foo" : "bar" } "#, {"foo": "bar"});
        map_test!(r#" { "123" : [] } "#, {"123": []});
        map_test!(r#" { "123" : [[], []] } "#, {"123": [[], []]});
        map_test!(
            r#" { "123" : [[true, false], [{}, {"foo": ["bar1","bar2"]}] ] } "#,
            {
                "123": [
                    [true, false],
                    [
                        {},
                        {"foo": ["bar1", "bar2"]}
                    ]
                ]
            }
        );

        Ok(())
    }

    #[test]
    fn parse_fileopts_chmod() -> ParseResult<()> {
        fileopts_test!("", (None, None));

        fileopts_test_err!("--chmod");
        fileopts_test_err!("--chmod ");
        fileopts_test_err!("--chmod=1");
        fileopts_test_err!("--chmod=12");
        fileopts_test_err!("--chmod=12345");

        fileopts_test!("--chmod=000", (None, Some(0)));
        fileopts_test!("--chmod=1234", (None, Some(0o1234)));
        fileopts_test!("--chmod=1750", (None, Some(0o1750)));
        fileopts_test!("--chmod=7777", (None, Some(0o7777)));

        fileopts_test!("--chmod \\\n 1750", (None, Some(0o1750)));

        Ok(())
    }

    #[test]
    fn parse_fileopts_chown() -> ParseResult<()> {
        fileopts_test_err!("--chown");
        fileopts_test_err!("--chown ");
        fileopts_test_err!("--chown:grp");

        fileopts_test_chown!("--chown :", None, None);

        fileopts_test_chown!("--chown user", Some("user"), None);
        fileopts_test_chown!("--chown=user", Some("user"), None);

        fileopts_test_chown!("--chown user:", Some("user"), Some("user"));
        fileopts_test_chown!("--chown :group", None, Some("group"));

        Ok(())
    }

    #[test]
    fn parse_fileopts_parent() -> ParseResult<()> {
        fileopts_test_full!("", None, None, None, false);
        fileopts_test_full!("-p", None, None, None, true);

        fileopts_test_full!("--chmod 1234 -p", Some(0o1234), None, None, true);
        fileopts_test_full!("-p --chmod 1234 -p", Some(0o1234), None, None, true);
        fileopts_test_full!("-p --chmod 1234", Some(0o1234), None, None, true);

        fileopts_test_full!("-p --chmod 1234 --chown user:group", Some(0o1234), Some("user"), Some("group"), true);

        Ok(())
    }
}
