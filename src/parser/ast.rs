use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use minijinja::Value;
use pest_consume::{match_nodes, Parser};

use crate::dsl::{
    Chown, IncludeArg, IncludeArgValue, InstCopy, InstEnv, InstEnvAssign, InstFrom, InstInclude,
    InstInvoke, InstRender, InstRun, InstWorkdir, InstWrite, Instruction, Lookup, Origin,
    Statement,
};
use crate::parser::{RaptorFileParser, Rule};
use crate::RaptorResult;

#[derive(Clone, Debug)]
pub struct UserData {
    pub path: Arc<Utf8PathBuf>,
}

type Result<T> = std::result::Result<T, pest_consume::Error<Rule>>;
pub type Node<'i> = pest_consume::Node<'i, Rule, UserData>;

#[allow(non_snake_case, clippy::unnecessary_wraps)]
#[pest_consume::parser]
impl RaptorFileParser {
    fn EOI(input: Node) -> Result<()> {
        Ok(())
    }

    fn COMMENT(input: Node) -> Result<()> {
        Ok(())
    }

    fn string_escape_seq(input: Node) -> Result<&str> {
        match input.as_str() {
            "\\t" => Ok("\t"),
            "\\n" => Ok("\n"),
            x => Err(input.error(format!("Unexpected ecsape: {x}"))),
        }
    }

    fn string_inner(input: Node) -> Result<String> {
        let mut res = String::new();

        for node in input.into_children() {
            match node.as_rule() {
                Rule::string_escape_seq => {
                    res += Self::string_escape_seq(node)?;
                }
                Rule::string_non_escape => {
                    res += node.as_str();
                }
                _ => {}
            }
        }
        Ok(res)
    }

    fn quoted_string(input: Node) -> Result<String> {
        Ok(match_nodes!(
            input.into_children();
            [string_inner(s)] => Ok(s),
        )?)
    }

    fn literal_string(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn user_spec(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn group_spec(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn string(input: Node) -> Result<String> {
        Ok(match_nodes!(
            input.into_children();
            [quoted_string(s)] => Ok(s),
            [literal_string(s)] => Ok(s),
        )?)
    }

    fn filename(input: Node) -> Result<String> {
        Ok(match_nodes!(
            input.into_children();
            [string(s)] => Ok(s),
            [literal_string(s)] => Ok(s),
        )?)
    }

    fn option_chown(input: Node) -> Result<Chown> {
        match_nodes!(
            input.into_children();
            [user_spec(user), group_spec(grp)] => Ok(Chown {
                user: Some(user),
                group: Some(grp),
            }),
            [group_spec(grp)] => Ok(Chown {
                user: None,
                group: Some(grp),
            }),
            [user_spec(user)] => Ok(Chown {
                user: Some(user),
                group: None,
            }),
        )
    }

    fn option_chmod(input: Node) -> Result<u32> {
        Ok(u32::from_str_radix(input.as_str(), 8).map_err(|e| input.error(e))?)
    }

    fn file_option(input: Node) -> Result<(Option<u32>, Option<Chown>)> {
        match_nodes!(
            input.into_children();
            [option_chown(chown)] => Ok((None, Some(chown))),
            [option_chmod(chmod)] => Ok((Some(chmod), None)),
        )
    }

    fn file_options(input: Node) -> Result<(Option<u32>, Option<Chown>)> {
        let mut chown = None;
        let mut chmod = None;

        let opts: Vec<_> = match_nodes!(
            input.into_children();
            [file_option(opt)..] => opt.collect()
        );

        for (opt_chmod, opt_chown) in opts {
            chown = opt_chown.or(chown);
            chmod = opt_chmod.or(chmod);
        }

        Ok((chmod, chown))
    }

    fn COPY(input: Node) -> Result<InstCopy> {
        let mut srcs: Vec<String>;
        let chmod;
        let chown;

        match_nodes!(
            input.into_children();
            [file_options(opts), filename(filenames)..] => {
                (chmod, chown) = opts;
                srcs = filenames.collect();
            },
        );

        let dest = srcs.pop().unwrap();

        Ok(InstCopy {
            srcs,
            dest,
            chmod,
            chown,
        })
    }

    fn RENDER(input: Node) -> Result<InstRender> {
        match_nodes!(
            input.into_children();
            [file_options((chmod, chown)), filename(src), filename(dest), include_args(args)] => {
                Ok(InstRender {
                    src,
                    dest,
                    chmod,
                    chown,
                    args
                })
            },
        )
    }

    fn WRITE(input: Node) -> Result<InstWrite> {
        match_nodes!(
            input.into_children();
            [file_options((chmod, chown)), filename(dest), quoted_string(body)] => {
                Ok(InstWrite {
                    dest,
                    body,
                    chmod,
                    chown,
                })
            },
        )
    }

    fn INVOKE(input: Node) -> Result<InstInvoke> {
        Ok(match_nodes!(input.into_children();
        [string(i)..] => InstInvoke {
            args: i.collect(),
        }))
    }

    fn ident(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn FROM(input: Node) -> Result<InstFrom> {
        Ok(match_nodes!(input.into_children();
        [ident(i)] => InstFrom {
            from: i,
        }))
    }

    fn RUN(input: Node) -> Result<InstRun> {
        Ok(match_nodes!(input.into_children();
        [filename(i)..] => InstRun {
            run: i.collect(),
        }))
    }

    fn bool(input: Node) -> Result<bool> {
        match input.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => todo!(),
        }
    }

    fn identpath(input: Node) -> Result<Value> {
        Ok(input.as_str().to_string().into())
    }

    fn number(input: Node) -> Result<i64> {
        Ok(input.as_str().parse::<i64>().map_err(|e| input.error(e))?)
    }

    fn value(input: Node) -> Result<IncludeArgValue> {
        let origin = Origin::from_node(&input);
        Ok(match_nodes!(
            input.into_children();
            [bool(b)] => IncludeArgValue::Value(b.into()),
            [number(b)] => IncludeArgValue::Value(b.into()),
            [string(b)] => IncludeArgValue::Value(b.into()),
            [ident(b)..] => IncludeArgValue::Lookup(Lookup::new(b.collect(), origin)),
        ))
    }

    fn include_arg(input: Node) -> Result<IncludeArg> {
        let origin = Origin::from_node(&input);
        Ok(match_nodes!(
            input.into_children();
            [ident(id), value(val)] => IncludeArg {
                name: id,
                value: val,
            },
            [ident(id)] => IncludeArg {
                name: id.clone(),
                value: IncludeArgValue::Lookup(Lookup::new(vec![id], origin)),
            }
        ))
    }

    fn include_args(input: Node) -> Result<Vec<IncludeArg>> {
        Ok(match_nodes!(input.into_children();
        [include_arg(args)..] => args.collect()))
    }

    fn env_assign(input: Node) -> Result<InstEnvAssign> {
        Ok(match_nodes!(input.into_children();
        [ident(key), string(value)] => InstEnvAssign {
            key,
            value,
        }))
    }

    fn ENV(input: Node) -> Result<InstEnv> {
        Ok(match_nodes!(input.into_children();
        [env_assign(res)..] => InstEnv {
            env: res.collect(),
        }))
    }

    fn WORKDIR(input: Node) -> Result<InstWorkdir> {
        Ok(match_nodes!(input.into_children();
        [filename(dir)] => InstWorkdir {
            dir
        }))
    }

    fn INCLUDE(input: Node) -> Result<InstInclude> {
        match_nodes!(
            input.into_children();
            [filename(src), include_args(args)] => {
                Ok(InstInclude {
                    src,
                    args,
                })
            },
        )
    }

    fn STATEMENT(input: Node) -> Result<Option<Statement>> {
        let origin = Origin::from_node(&input);
        Ok(match_nodes!(
            input.into_children();
            [FROM(stmt)] => Some(Statement { inst: Instruction::From(stmt), origin }),
            [COPY(stmt)] => Some(Statement { inst: Instruction::Copy(stmt), origin }),
            [WRITE(stmt)] => Some(Statement { inst: Instruction::Write(stmt), origin }),
            [RENDER(stmt)] => Some(Statement { inst: Instruction::Render(stmt), origin }),
            [INCLUDE(stmt)] => Some(Statement { inst: Instruction::Include(stmt), origin }),
            [INVOKE(stmt)] => Some(Statement { inst: Instruction::Invoke(stmt), origin }),
            [RUN(stmt)] => Some(Statement { inst: Instruction::Run(stmt), origin }),
            [ENV(stmt)] => Some(Statement { inst: Instruction::Env(stmt), origin }),
            [WORKDIR(stmt)] => Some(Statement { inst: Instruction::Workdir(stmt), origin }),
            [] => None,
        ))
    }

    fn FILE(input: Node) -> Result<Vec<Statement>> {
        match_nodes!(
            input.into_children();
            [STATEMENT(stmt).., _EOI] => Ok(stmt.flatten().collect())
        )
    }
}

pub fn parse(path: impl AsRef<Utf8Path>, input: &str) -> RaptorResult<Vec<Statement>> {
    let inputs = RaptorFileParser::parse_with_userdata(
        Rule::FILE,
        input,
        UserData {
            path: Arc::new(path.as_ref().into()),
        },
    )?;
    let input = inputs.single()?;
    Ok(RaptorFileParser::FILE(input)?)
}
