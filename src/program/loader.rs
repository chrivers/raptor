use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use colored::Colorize;
use minijinja::{Environment, ErrorKind, Value, context};

use crate::dsl::{Item, Program};
use crate::program::{
    show_error_context, show_jinja_error_context, show_origin_error_context,
    show_pest_error_context,
};
use crate::template::make_environment;
use crate::{RaptorError, RaptorResult};
use raptor_parser::ast::{Instruction, Origin, ResolveArgs, Statement};
use raptor_parser::{ParseError, parser};

pub struct Loader<'source> {
    env: Environment<'source>,
    dump: bool,
    sources: HashMap<String, String>,
    base: Utf8PathBuf,
    origins: Vec<Origin>,
}

const MAX_NESTED_INCLUDE: usize = 20;

impl Loader<'_> {
    pub fn new() -> RaptorResult<Self> {
        Ok(Self {
            env: make_environment()?,
            dump: false,
            base: Utf8PathBuf::new(),
            sources: HashMap::new(),
            origins: vec![],
        })
    }

    #[must_use]
    pub fn with_base(self, base: impl AsRef<Utf8Path>) -> Self {
        Self {
            base: base.as_ref().into(),
            ..self
        }
    }

    #[must_use]
    pub fn with_dump(self, dump: bool) -> Self {
        Self { dump, ..self }
    }

    pub fn base(&self) -> &Utf8Path {
        &self.base
    }

    pub fn clear_cache(&mut self) {
        self.env.clear_templates();
    }

    fn handle(&mut self, res: &mut Vec<Item>, stmt: Statement, ctx: &Value) -> RaptorResult<()> {
        let Statement { inst, origin } = stmt;

        if let Instruction::Include(include) = &inst {
            if self.origins.len() >= MAX_NESTED_INCLUDE {
                return Err(RaptorError::ScriptError(
                    "Too many nested includes".into(),
                    self.origins.last().unwrap().clone(),
                ));
            }

            let map = include.args.clone().resolve_args(ctx)?;
            let src = &origin.basedir()?.join(include.src.to_include_path());

            self.origins.push(origin.clone());
            let program = self.parse_template(src, Value::from(map))?;
            self.origins.pop();

            res.push(Item::Statement(Statement { inst, origin }));
            res.push(Item::Program(program));
        } else {
            res.push(Item::Statement(Statement { inst, origin }));
        }

        Ok(())
    }

    fn show_include_stack(&self, origins: &[Origin]) {
        for org in origins {
            show_origin_error_context(
                &self.sources[org.path.as_str()],
                org,
                "Error while evaluating INCLUDE",
                "(included here)",
            );
        }
    }

    pub fn explain_error(&self, err: &RaptorError) -> RaptorResult<()> {
        match err {
            RaptorError::ScriptError(_, origin)
            | RaptorError::ParseError(ParseError::UndefinedVarError(_, origin)) => {
                self.show_include_stack(&self.origins);
                show_origin_error_context(
                    &self.sources[origin.path.as_str()],
                    origin,
                    "Script Error",
                    &err.to_string(),
                );
            }
            RaptorError::MinijinjaError(err) => {
                if err.kind() == ErrorKind::BadInclude {
                    if let Some((last, origins)) = &self.origins.split_last() {
                        self.show_include_stack(origins);

                        show_error_context(
                            &self.sources[last.path.as_str()],
                            last.path.as_ref(),
                            "Error while evaluating INCLUDE",
                            err.detail().unwrap_or("error"),
                            err.range().unwrap_or_else(|| last.span.clone()),
                        );
                    } else {
                        error!("Cannot provide error context: {err}");
                    }
                } else {
                    self.show_include_stack(&self.origins);
                    show_jinja_error_context(err)?;
                    let mut err = &err as &dyn std::error::Error;
                    while let Some(next_err) = err.source() {
                        error!("{}\n{:#}", "caused by:".bright_white(), next_err);
                        err = next_err;
                    }
                }
            }
            RaptorError::ParseError(ParseError::PestError(err)) => {
                show_pest_error_context(&self.sources[err.path().unwrap()], err)?;
            }
            err => {
                error!("Unexpected error: {err}");
            }
        }
        Ok(())
    }

    pub fn explain_exec_error(&self, stmt: &Statement, err: &RaptorError) -> RaptorResult<()> {
        let origin = &stmt.origin;

        let prefix = err.category();

        show_origin_error_context(
            &self.sources[origin.path.as_str()],
            origin,
            "Error while executing instruction",
            &format!("{prefix}: {err}"),
        );

        if let RaptorError::MinijinjaError(_) = err {
            self.explain_error(err)?;
        }

        Ok(())
    }

    pub fn origins(&self) -> &[Origin] {
        &self.origins
    }

    pub fn parse_template(
        &mut self,
        path: impl AsRef<Utf8Path>,
        ctx: Value,
    ) -> RaptorResult<Program> {
        let tmpl = self.env.get_template(self.base.join(&path).as_str())?;
        let (source, state) = tmpl
            .render_and_return_state(ctx.clone())
            .map(|(src, state)| (src + "\n", state))?;

        let exports = state
            .exports()
            .into_iter()
            .map(|key| (key, state.lookup(key).unwrap()))
            .collect::<Value>();

        let ctx = context! {
            ..exports,
            ..ctx,
        };

        if self.dump {
            info!("Template output for [{}]", path.as_ref());
            println!("{}\n", source.trim_end());
        }

        let filename = path.as_ref().as_str();

        self.sources.insert(filename.into(), source);

        let statements =
            parser::parse(filename, &self.sources[filename]).map_err(|err| match err {
                ParseError::PestError(err) => {
                    ParseError::PestError(Box::new(err.with_path(filename)))
                }
                err => err,
            })?;

        let mut res = vec![];

        for stmt in statements {
            self.handle(&mut res, stmt, &ctx)?;
        }

        Ok(Program::new(res, ctx, path.as_ref().into()))
    }
}
