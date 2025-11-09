use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use colored::Colorize;
use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use minijinja::{Environment, ErrorKind, Value, context};
use raptor_parser::ast::{Instruction, Origin, Statement};
use raptor_parser::parser;
use raptor_parser::util::module_name::{ModuleName, ModuleRoot};

use crate::dsl::{Item, Program};
use crate::program::{
    ResolveArgs, show_error_context, show_jinja_error_context, show_origin_error_context,
    show_parse_error_context,
};
use crate::template::make_environment;
use crate::{RaptorError, RaptorResult};

pub struct Loader<'source> {
    env: Environment<'source>,
    dump: bool,
    sources: DashMap<String, String>,
    base: Utf8PathBuf,
    packages: DashMap<String, Utf8PathBuf>,
    programs: DashMap<Utf8PathBuf, Arc<Program>>,
}

const MAX_NESTED_INCLUDE: usize = 20;

impl Loader<'_> {
    pub fn new() -> RaptorResult<Self> {
        Ok(Self {
            env: make_environment()?,
            dump: false,
            base: Utf8PathBuf::new(),
            sources: DashMap::new(),
            packages: DashMap::new(),
            programs: DashMap::new(),
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

    pub fn add_package(&self, name: String, path: Utf8PathBuf) {
        self.packages.insert(name, path);
    }

    pub fn get_package(&self, name: &str) -> Option<Ref<'_, String, Utf8PathBuf>> {
        self.packages.get(name)
    }

    fn to_path(
        &self,
        root: &ModuleRoot,
        origin: &Origin,
        end: &Utf8Path,
    ) -> RaptorResult<Utf8PathBuf> {
        let res = match root {
            ModuleRoot::Relative => origin.path_for(end)?,
            ModuleRoot::Absolute => self.base.join(end),
            ModuleRoot::Package(pkg) => {
                let package = self
                    .get_package(pkg)
                    .ok_or_else(|| RaptorError::PackageNotFound(pkg.clone(), origin.clone()))?;
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
        self.sources.clear();
        self.programs.clear();
    }

    fn handle(
        &self,
        prog: &mut Program,
        origins: &mut Vec<Origin>,
        stmt: Statement,
    ) -> RaptorResult<()> {
        let Statement { inst, origin } = stmt;

        if let Instruction::Include(include) = &inst {
            if origins.len() >= MAX_NESTED_INCLUDE {
                return Err(RaptorError::ScriptError(
                    "Too many nested includes".into(),
                    origins.last().unwrap().clone(),
                ));
            }

            let mut context = Value::from(prog.ctx.resolve_args(&include.args)?);
            let src = self.to_include_path(&include.src, &origin)?;

            if let Some(instance) = include.src.instance() {
                context = context! { instance, ..context };
            }

            origins.push(origin.clone());
            let include = self.parse_template(&src, origins, context)?;
            origins.pop();

            prog.code.push(Item::Statement(Statement { inst, origin }));
            prog.code.push(Item::Program(include));
        } else {
            prog.code.push(Item::Statement(Statement { inst, origin }));
        }

        Ok(())
    }

    fn show_include_stack(&self, origins: &[Origin]) {
        for org in origins {
            let Some(source) = self.sources.get(org.path.as_str()) else {
                continue;
            };
            show_origin_error_context(
                &source,
                org,
                "Error while evaluating INCLUDE",
                "(included here)",
            );
        }
    }

    pub fn explain_error(&self, err: &RaptorError, origins: &[Origin]) -> RaptorResult<()> {
        match err {
            RaptorError::ScriptError(_, origin) | RaptorError::UndefinedVarError(_, origin) => {
                self.show_include_stack(origins);
                show_origin_error_context(
                    &self.sources.get(origin.path.as_str()).unwrap(),
                    origin,
                    "Script Error",
                    &err.to_string(),
                );
            }
            RaptorError::MinijinjaError(err) => {
                if err.kind() == ErrorKind::BadInclude {
                    if let Some((last, origins)) = &origins.split_last() {
                        self.show_include_stack(origins);

                        if let Some(src) = self.sources.get(last.path.as_str()) {
                            show_error_context(
                                &src,
                                last.path.as_ref(),
                                "Error while loading source file",
                                err.detail().unwrap_or("error"),
                                err.range().unwrap_or_else(|| last.span.clone()),
                            );
                        }
                    } else {
                        error!("Cannot provide error context: {err}");
                    }
                } else {
                    self.show_include_stack(origins);
                    show_jinja_error_context(err)?;
                    let mut err = &err as &dyn std::error::Error;
                    while let Some(next_err) = err.source() {
                        error!("{}\n{:#}", "caused by:".bright_white(), next_err);
                        err = next_err;
                    }
                }
            }
            RaptorError::ParseError(err) => {
                show_parse_error_context(
                    &self.sources.get(err.origin.path.as_str()).unwrap(),
                    err,
                )?;
            }
            RaptorError::PackageNotFound(pkg, origin) => {
                self.show_include_stack(origins);
                show_origin_error_context(
                    &self.sources.get(origin.path.as_str()).unwrap(),
                    origin,
                    &format!("Package not found: ${pkg}"),
                    &err.to_string(),
                );
            }
            RaptorError::SandboxRequestError(_errno) => {
                if let Some((last, origins)) = &origins.split_last() {
                    self.show_include_stack(origins);

                    let prefix = err.category();
                    show_origin_error_context(
                        &self.sources.get(last.path.as_str()).unwrap(),
                        last,
                        "Error while executing instruction",
                        &format!("{prefix}: {err}"),
                    );
                }
            }
            err => {
                error!("Unexpected error: {err}");
            }
        }
        Ok(())
    }

    fn parse_template(
        &self,
        path: impl AsRef<Utf8Path>,
        origins: &mut Vec<Origin>,
        ctx: Value,
    ) -> RaptorResult<Arc<Program>> {
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

        let statements = parser::parse(filename, &self.sources.get(filename).unwrap())?;

        let mut program = Program::new(vec![], ctx, path.as_ref().into());

        for stmt in statements {
            self.handle(&mut program, origins, stmt)?;
        }

        Ok(Arc::new(program))
    }

    pub fn load_template(
        &self,
        path: impl AsRef<Utf8Path>,
        ctx: Value,
        origins: &mut Vec<Origin>,
    ) -> RaptorResult<Arc<Program>> {
        let path = path.as_ref();

        if let Some(program) = self.programs.get(path) {
            return Ok(program.clone());
        }

        let program = self.parse_template(path, origins, ctx)?;

        self.programs.insert(path.into(), program);

        Ok(self.programs.get(path).unwrap().clone())
    }

    pub fn load_program(&self, name: &ModuleName, origin: Origin) -> RaptorResult<Arc<Program>> {
        let path = self.to_program_path(name, &origin)?;
        let context = name
            .instance()
            .as_ref()
            .map_or_else(|| context! {}, |instance| context! { instance });

        let mut origins = vec![origin];

        self.load_template(&path, context, &mut origins)
            .or_else(|err| {
                self.explain_error(&err, &origins)?;
                Err(err)
            })
    }
}
