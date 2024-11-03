use pest_consume::{match_nodes, Parser};

use crate::dsl::{Chown, InstCopy, InstFrom, InstRender, Instruction};
use crate::parser::{RaptorFileParser, Rule};
use crate::RaptorResult;

type Result<T> = std::result::Result<T, pest_consume::Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[allow(non_snake_case, clippy::unnecessary_wraps)]
#[pest_consume::parser]
impl RaptorFileParser {
    fn EOI(input: Node) -> Result<()> {
        Ok(())
    }

    fn COMMENT(input: Node) -> Result<()> {
        Ok(())
    }

    fn string(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
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

    fn copy_option(input: Node) -> Result<(Option<u16>, Option<Chown>)> {
        match_nodes!(
            input.into_children();
            [option_chown(chown)] => Ok((None, Some(chown))),
            [option_chmod(chmod)] => Ok((Some(chmod), None)),
        )
    }

    fn copy_options(input: Node) -> Result<(Option<u16>, Option<Chown>)> {
        let mut chown = None;
        let mut chmod = None;

        let opts: Vec<_> = match_nodes!(
            input.into_children();
            [copy_option(opt)..] => opt.collect()
        );

        for (opt_chmod, opt_chown) in opts {
            chown = opt_chown.or(chown);
            chmod = opt_chmod.or(chmod);
        }

        Ok((chmod, chown))
    }

    fn render_options(input: Node) -> Result<(Option<u16>, Option<Chown>)> {
        let mut chown = None;
        let mut chmod = None;

        let opts: Vec<_> = match_nodes!(
            input.into_children();
            [copy_option(opt)..] => opt.collect()
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
            [copy_options(opts), filename(filenames)..] => {
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
            [render_options((chmod, chown)), filename(src), filename(dest)] => {
                Ok(InstRender {
                    src,
                    dest,
                    chmod,
                    chown,
                })
            },
        )
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

    fn STATEMENT(input: Node) -> Result<Option<Instruction>> {
        Ok(match_nodes!(
            input.into_children();
            [FROM(stmt)] => Some(Instruction::From(stmt)),
            [COPY(stmt)] => Some(Instruction::Copy(stmt)),
            [RENDER(stmt)] => Some(Instruction::Render(stmt)),
            [] => None,
        ))
    }

    fn FILE(input: Node) -> Result<Vec<Instruction>> {
        match_nodes!(
            input.into_children();
            [STATEMENT(stmt).., _EOI] => Ok(stmt.flatten().collect())
        )
    }
}

pub fn parse(input: &str) -> RaptorResult<Vec<Instruction>> {
    let inputs = RaptorFileParser::parse(Rule::FILE, input)?;
    let input = inputs.single()?;
    Ok(RaptorFileParser::FILE(input)?)
}
