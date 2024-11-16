use std::sync::Arc;

use minijinja::Value;
use pest_consume::{match_nodes, Parser};

use crate::dsl::{
    Chown, IncludeArg, IncludeArgValue, InstCopy, InstFrom, InstInclude, InstInvoke, InstRender,
    InstRun, InstWrite, Instruction, Lookup, Origin, Statement,
};
use crate::parser::{RaptorFileParser, Rule};
use crate::RaptorResult;

#[derive(Clone, Debug)]
struct UserData {
    path: Arc<String>,
}

type Result<T> = std::result::Result<T, pest_consume::Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, UserData>;

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

    fn string(input: Node) -> Result<String> {
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

    fn literal_string(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn user_spec(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn group_spec(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
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

    fn option_chmod(input: Node) -> Result<u16> {
        Ok(u16::from_str_radix(input.as_str(), 8).map_err(|e| input.error(e))?)
    }

    fn file_option(input: Node) -> Result<(Option<u16>, Option<Chown>)> {
        match_nodes!(
            input.into_children();
            [option_chown(chown)] => Ok((None, Some(chown))),
            [option_chmod(chmod)] => Ok((Some(chmod), None)),
        )
    }

    fn file_options(input: Node) -> Result<(Option<u16>, Option<Chown>)> {
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
            [file_options((chmod, chown)), filename(src), filename(dest)] => {
                Ok(InstRender {
                    src,
                    dest,
                    chmod,
                    chown,
                })
            },
        )
    }

    fn WRITE(input: Node) -> Result<InstWrite> {
        match_nodes!(
            input.into_children();
            [file_options((chmod, chown)), filename(dest), string(body)] => {
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
        Ok(match_nodes!(
            input.into_children();
            [bool(b)] => IncludeArgValue::Value(b.into()),
            [number(b)] => IncludeArgValue::Value(b.into()),
            [string(b)] => IncludeArgValue::Value(b.into()),
            [ident(b)..] => IncludeArgValue::Lookup(Lookup::new(b.collect())),
        ))
    }

    fn include_arg(input: Node) -> Result<IncludeArg> {
        Ok(match_nodes!(
            input.into_children();
            [ident(id), value(val)] => IncludeArg {
                name: id,
                value: val,
            }
        ))
    }

    fn include_args(input: Node) -> Result<Vec<IncludeArg>> {
        Ok(match_nodes!(input.into_children();
        [include_arg(args)..] => args.collect()))
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
        let span = input.as_span();
        let origin = Origin {
            path: input.user_data().path.clone(),
            span: span.start()..span.end(),
        };
        Ok(match_nodes!(
            input.into_children();
            [FROM(stmt)] => Some(Statement { inst: Instruction::From(stmt), origin }),
            [COPY(stmt)] => Some(Statement { inst: Instruction::Copy(stmt), origin }),
            [WRITE(stmt)] => Some(Statement { inst: Instruction::Write(stmt), origin }),
            [RENDER(stmt)] => Some(Statement { inst: Instruction::Render(stmt), origin }),
            [INCLUDE(stmt)] => Some(Statement { inst: Instruction::Include(stmt), origin }),
            [INVOKE(stmt)] => Some(Statement { inst: Instruction::Invoke(stmt), origin }),
            [RUN(stmt)] => Some(Statement { inst: Instruction::Run(stmt), origin }),
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

pub fn parse(path: &str, input: &str) -> RaptorResult<Vec<Statement>> {
    let inputs = RaptorFileParser::parse_with_userdata(
        Rule::FILE,
        input,
        UserData {
            path: Arc::new(path.to_string()),
        },
    )?;
    let input = inputs.single()?;
    Ok(RaptorFileParser::FILE(input)?)
}
