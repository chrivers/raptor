use std::collections::HashMap;

use minijinja::{Environment, ErrorKind, Value};

use crate::dsl::{IncludeArgValue, Instruction, Origin, Statement};
use crate::parser::ast;
use crate::program::{show_error_context, show_jinja_error_context, show_pest_error_context};
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

    fn handle(&mut self, stmt: Statement, rctx: &Value) -> RaptorResult<Vec<Statement>> {
        if let Instruction::Include(inst) = &stmt.inst {
            if self.origins.len() >= MAX_NESTED_INCLUDE {
                return Err(RaptorError::ScriptError(
                    "Too many nested includes".to_string(),
                    self.origins.last().unwrap().clone(),
                ));
            }
            let mut map = HashMap::new();
            for arg in &inst.args {
                match &arg.value {
                    IncludeArgValue::Lookup(lookup) => {
                        let name = &lookup.path[0];
                        let val = rctx.get_attr(name)?;
                        if val.is_undefined() {
                            Err(RaptorError::ScriptError(
                                format!("Undefined variable {name:?}"),
                                lookup.origin.clone(),
                            ))?;
                        }
                        map.insert(arg.name.clone(), val.clone());
                    }
                    IncludeArgValue::Value(val) => {
                        map.insert(arg.name.clone(), Value::from_serialize(val));
                    }
                }
            }

            let ctx = Value::from(map);
            self.origins.push(stmt.origin.clone());
            let statements = self.parse_template(&inst.src, &ctx)?;
            self.origins.pop();

            Ok(statements)
        } else {
            Ok(vec![stmt])
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

    pub fn explain_error(&self, err: RaptorError) -> RaptorResult<()> {
        match err {
            RaptorError::ScriptError(desc, origin) => {
                self.show_include_stack(&self.origins);
                show_error_context(
                    &self.sources[origin.path.as_str()],
                    &origin.path,
                    "Script Error",
                    &desc,
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
                    show_jinja_error_context(&err)?;
                }
            }
            RaptorError::PestError(err) => {
                show_pest_error_context(&self.sources[err.path().unwrap()], &err)?;
            }
            err => {
                error!("Unexpected error: {err}");
            }
        }
        Ok(())
    }

    pub fn origins(&self) -> &[Origin] {
        &self.origins
    }

    pub fn parse_template(&mut self, path: &str, ctx: &Value) -> RaptorResult<Vec<Statement>> {
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
            res.extend(self.handle(stmt, ctx)?);
        }

        Ok(res)
    }
}
