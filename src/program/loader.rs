use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use colored::Colorize;
use minijinja::{Environment, ErrorKind, Value};

use crate::dsl::{Instruction, Item, Origin, Program, ResolveArgs, Statement};
use crate::parser::ast;
use crate::program::{
    show_error_context, show_jinja_error_context, show_origin_error_context,
    show_pest_error_context,
};
use crate::template::make_environment;
use crate::{RaptorError, RaptorResult};

pub struct Loader<'source> {
    env: Environment<'source>,
    dump: bool,
    sources: HashMap<String, String>,
    base: Utf8PathBuf,
    origins: Vec<Origin>,
}

const MAX_NESTED_INCLUDE: usize = 20;

impl<'source> Loader<'source> {
    pub fn new(base: impl AsRef<Utf8Path>, dump: bool) -> RaptorResult<Self> {
        Ok(Self {
            env: make_environment()?,
            dump,
            base: base.as_ref().into(),
            sources: HashMap::new(),
            origins: vec![],
        })
    }

    pub fn base(&self) -> &Utf8Path {
        &self.base
    }

    fn handle(&mut self, stmt: Statement, rctx: &Value) -> RaptorResult<Item> {
        let Statement { inst, origin } = stmt;

        if let Instruction::Include(inst) = inst {
            if self.origins.len() >= MAX_NESTED_INCLUDE {
                return Err(RaptorError::ScriptError(
                    "Too many nested includes".into(),
                    self.origins.last().unwrap().clone(),
                ));
            }

            let map = inst.args.resolve_args(rctx)?;
            let src = &origin.basedir()?.join(inst.src);

            self.origins.push(origin);
            let program = self.parse_template(src, &Value::from(map))?;
            self.origins.pop();

            Ok(Item::Program(program))
        } else {
            Ok(Item::Statement(Statement { inst, origin }))
        }
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
            RaptorError::ScriptError(desc, origin)
            | RaptorError::UndefinedVarError(desc, origin) => {
                self.show_include_stack(&self.origins);
                show_origin_error_context(
                    &self.sources[origin.path.as_str()],
                    origin,
                    "Script Error",
                    desc,
                );
            }
            RaptorError::MinijinjaError(err) => {
                let mut origins = self.origins().to_vec();
                if err.kind() == ErrorKind::BadInclude {
                    if let Some(last) = origins.pop() {
                        self.show_include_stack(&origins);

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
                    self.show_include_stack(&origins);
                    show_jinja_error_context(err)?;
                    let mut err = &err as &dyn std::error::Error;
                    while let Some(next_err) = err.source() {
                        error!("{}\n{:#}", "caused by:".bright_white(), next_err);
                        err = next_err;
                    }
                }
            }
            RaptorError::PestError(err) => {
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
        ctx: &Value,
    ) -> RaptorResult<Program> {
        let source = self
            .env
            .get_template(self.base.join(&path).as_str())
            .and_then(|tmpl| tmpl.render(ctx))
            .map(|src| src + "\n")?;

        if self.dump {
            println!("{source}");
        }

        let filename = path.as_ref().as_str();

        self.sources.insert(filename.into(), source);

        let mut res = vec![];

        for stmt in ast::parse(filename, &self.sources[filename]).map_err(|err| match err {
            RaptorError::PestError(err) => {
                RaptorError::PestError(Box::new(err.with_path(filename)))
            }
            err => err,
        })? {
            res.push(self.handle(stmt, ctx)?);
        }

        Ok(Program::new(res, ctx.clone(), path.as_ref().into()))
    }
}
