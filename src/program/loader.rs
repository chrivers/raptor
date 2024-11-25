use std::collections::HashMap;

use minijinja::{Environment, ErrorKind, Value};

use crate::dsl::{InstInclude, Instruction, Item, Origin, ResolveArgs, Statement};
use crate::parser::ast;
use crate::program::{
    show_error_context, show_jinja_error_context, show_pest_error_context, Program,
};
use crate::{RaptorError, RaptorResult};

pub struct Loader<'source> {
    env: Environment<'source>,
    dump: bool,
    sources: HashMap<String, String>,
    origins: Vec<Origin>,
}

const MAX_NESTED_INCLUDE: usize = 20;

impl<'source> Loader<'source> {
    pub fn new(env: Environment<'source>, dump: bool) -> Self {
        Self {
            env,
            dump,
            sources: HashMap::new(),
            origins: vec![],
        }
    }

    fn handle(&mut self, stmt: Statement, rctx: &Value) -> RaptorResult<Item> {
        if let Instruction::Include(InstInclude { src, args }) = stmt.inst {
            if self.origins.len() >= MAX_NESTED_INCLUDE {
                return Err(RaptorError::ScriptError(
                    "Too many nested includes".into(),
                    self.origins.last().unwrap().clone(),
                ));
            }

            let map = args.resolve_args(rctx)?;

            self.origins.push(stmt.origin);
            let program = self.parse_template(&src, &Value::from(map))?;
            self.origins.pop();

            Ok(Item::Program(program))
        } else {
            Ok(Item::Statement(stmt))
        }
    }

    fn show_include_stack(&self, origins: &[Origin]) {
        for org in origins {
            show_error_context(
                &self.sources[org.path.as_str()],
                &org.path,
                "Error while evaluating INCLUDE",
                "(included here)",
                org.span.clone(),
            );
        }
    }

    pub fn explain_error(&self, err: &RaptorError) -> RaptorResult<()> {
        match err {
            RaptorError::ScriptError(desc, origin) => {
                self.show_include_stack(&self.origins);
                show_error_context(
                    &self.sources[origin.path.as_str()],
                    &origin.path,
                    "Script Error",
                    desc,
                    origin.span.clone(),
                );
            }
            RaptorError::MinijinjaError(err) => {
                let mut origins = self.origins().to_vec();
                if err.kind() == ErrorKind::BadInclude {
                    if let Some(last) = origins.pop() {
                        self.show_include_stack(&origins);

                        show_error_context(
                            &self.sources[last.path.as_str()],
                            &last.path,
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

        let prefix = match err {
            RaptorError::IoError(_) => "IO Error",
            RaptorError::MinijinjaError(_) => "Template error",
            RaptorError::PestError(_) => "Parser error",
            RaptorError::BincodeError(_) => "Encoding error",
            RaptorError::VarError(_) => "Environment error",
            RaptorError::Errno(_) => "Errno",
            RaptorError::BadPath(_) => "Path encoding error",
            RaptorError::ScriptError(_, _) => "Script error",
            RaptorError::UndefinedVarError(_, _) => "Undefined variable",
            RaptorError::SandboxRequestError(_) => "Sandbox request error",
            RaptorError::SandboxRunError(_) => "Sandbox run error",
            RaptorError::MpscTimeout(_) => "Channel error",
            RaptorError::SendError(_) => "Send error",
        };

        show_error_context(
            &self.sources[origin.path.as_str()],
            &origin.path,
            "Error while executing instruction",
            &format!("{prefix}: {err}"),
            origin.span.clone(),
        );

        if let RaptorError::MinijinjaError(_) = err {
            self.explain_error(err)?;
        }

        Ok(())
    }

    pub fn origins(&self) -> &[Origin] {
        &self.origins
    }

    pub fn parse_template(&mut self, path: &str, ctx: &Value) -> RaptorResult<Program> {
        let source = self
            .env
            .get_template(path)
            .and_then(|tmpl| tmpl.render(ctx))
            .map(|src| src + "\n")?;

        if self.dump {
            println!("{source}");
        }

        self.sources.insert(path.to_string(), source);

        let mut res = vec![];

        for stmt in ast::parse(path, &self.sources[path]).map_err(|err| match err {
            RaptorError::PestError(err) => RaptorError::PestError(Box::new(err.with_path(path))),
            err => err,
        })? {
            res.push(self.handle(stmt, ctx)?);
        }

        Ok(Program::new(res, ctx.clone()))
    }
}
