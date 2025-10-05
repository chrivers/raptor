use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use colored::Colorize;
use minijinja::{Environment, ErrorKind, Value, context};
use raptor_parser::util::SafeParent;
use raptor_parser::util::module_name::{ModuleName, ModuleRoot};

use crate::dsl::{Item, Program};
use crate::program::{
    ResolveArgs, show_error_context, show_jinja_error_context, show_origin_error_context,
    show_parse_error_context,
};
use crate::template::make_environment;
use crate::{RaptorError, RaptorResult};
use raptor_parser::ast::{Instruction, Origin, Statement};
use raptor_parser::parser;

pub struct Loader<'source> {
    env: Environment<'source>,
    dump: bool,
    sources: HashMap<String, String>,
    base: Utf8PathBuf,
    origins: Vec<Origin>,
    packages: HashMap<String, Utf8PathBuf>,
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
            packages: HashMap::new(),
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

    pub fn add_package(&mut self, name: String, path: Utf8PathBuf) {
        self.packages.insert(name, path);
    }

    pub fn get_package(&self, name: &str) -> Option<&Utf8PathBuf> {
        self.packages.get(name)
    }

    pub fn push_origin(&mut self, origin: Origin) {
        self.origins.push(origin);
    }

    pub fn pop_origin(&mut self) {
        self.origins.pop();
    }

    pub fn to_path(
        &self,
        root: &ModuleRoot,
        origin: &Origin,
        end: &Utf8Path,
    ) -> RaptorResult<Utf8PathBuf> {
        let res = match root {
            ModuleRoot::Relative => origin.path.try_parent()?.join(end),
            ModuleRoot::Absolute => self.base.join(end),
            ModuleRoot::Package(pkg) => {
                let package = self
                    .get_package(pkg)
                    .ok_or_else(|| RaptorError::PackageNotFound(pkg.to_string(), origin.clone()))?;
                package.join(end)
            }
        };

        Ok(res)
    }

    pub fn to_program_path(&self, name: &ModuleName, origin: &Origin) -> RaptorResult<Utf8PathBuf> {
        let mut end = Utf8PathBuf::new();
        end.extend(name.parts());
        end.set_extension("rapt");
        self.to_path(name.root(), origin, &end)
    }

    pub fn to_include_path(&self, name: &ModuleName, origin: &Origin) -> RaptorResult<Utf8PathBuf> {
        let mut end = Utf8PathBuf::new();
        end.extend(name.parts());
        end.set_extension("rinc");
        self.to_path(name.root(), origin, &end)
    }

    pub fn base(&self) -> &Utf8Path {
        &self.base
    }

    pub fn clear_cache(&mut self) {
        self.env.clear_templates();
    }

    fn handle(&mut self, prog: &mut Program, stmt: Statement) -> RaptorResult<()> {
        let Statement { inst, origin } = stmt;

        if let Instruction::Include(include) = &inst {
            if self.origins.len() >= MAX_NESTED_INCLUDE {
                return Err(RaptorError::ScriptError(
                    "Too many nested includes".into(),
                    self.origins.last().unwrap().clone(),
                ));
            }

            let map = prog.ctx.resolve_args(&include.args)?;
            let src = self.to_include_path(&include.src, &origin)?;

            self.origins.push(origin.clone());
            let include = self.parse_template(src, Value::from(map))?;
            self.origins.pop();

            prog.code.push(Item::Statement(Statement { inst, origin }));
            prog.code.push(Item::Program(include));
        } else {
            prog.code.push(Item::Statement(Statement { inst, origin }));
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
            RaptorError::ScriptError(_, origin) | RaptorError::UndefinedVarError(_, origin) => {
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
                            "Error while loading source file",
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
            RaptorError::ParseError(err) => {
                show_parse_error_context(&self.sources[err.origin.path.as_str()], err)?;
            }
            RaptorError::PackageNotFound(pkg, origin) => {
                self.show_include_stack(&self.origins);
                show_origin_error_context(
                    &self.sources[origin.path.as_str()],
                    origin,
                    &format!("Package not found: ${pkg}"),
                    &err.to_string(),
                );
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

        let statements = parser::parse(filename, &self.sources[filename])?;

        let mut program = Program::new(vec![], ctx, path.as_ref().into());

        for stmt in statements {
            self.handle(&mut program, stmt)?;
        }

        Ok(program)
    }
}
