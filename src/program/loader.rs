use std::collections::HashMap;

use minijinja::{Environment, Value};

use crate::dsl::{IncludeArgValue, Instruction, Origin, Statement};
use crate::parser::ast;
use crate::{RaptorError, RaptorResult};

pub struct Loader<'source> {
    env: Environment<'source>,
    dump: bool,
    origins: Vec<Origin>,
}

const MAX_NESTED_INCLUDE: usize = 20;

impl<'source> Loader<'source> {
    pub const fn new(env: Environment<'source>, dump: bool) -> Self {
        Self {
            env,
            dump,
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

        let mut res = vec![];

        for stmt in ast::parse(path, &source)? {
            res.extend(self.handle(stmt, ctx)?);
        }

        Ok(res)
    }
}
