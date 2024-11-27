use std::collections::HashMap;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use colored::Colorize;
use minijinja::context;

use crate::build::Cacher;
use crate::dsl::Program;
use crate::program::{Executor, Loader, PrintExecutor};
use crate::sandbox::Sandbox;
use crate::util::SafeParent;
use crate::RaptorResult;

pub struct RaptorBuilder<'a> {
    loader: Loader<'a>,
    dry_run: bool,
    programs: HashMap<Utf8PathBuf, Arc<Program>>,
}

impl<'a> RaptorBuilder<'a> {
    pub fn new(loader: Loader<'a>, dry_run: bool) -> Self {
        Self {
            loader,
            dry_run,
            programs: HashMap::new(),
        }
    }

    pub fn load(&mut self, path: impl AsRef<Utf8Path>) -> RaptorResult<Arc<Program>> {
        let program = match self.loader.parse_template(&path, context! {}) {
            Ok(res) => res,
            Err(err) => {
                self.loader.explain_error(&err)?;
                return Err(err);
            }
        };

        let key = path.as_ref().into();

        let res = self
            .programs
            .entry(key)
            .or_insert_with(|| Arc::new(program));
        Ok(res.clone())
    }

    pub fn recurse(
        &mut self,
        program: Arc<Program>,
        visitor: &mut impl FnMut(Arc<Program>),
    ) -> RaptorResult<()> {
        if let Some(from) = program.from().map(|from| format!("{from}.rapt")) {
            let base = program.path.try_parent()?;

            let filename = base.join(from);
            let fromprog = self.load(filename)?;

            self.recurse(fromprog, visitor)?;
        }

        visitor(program);

        Ok(())
    }

    pub fn stack(&mut self, program: Arc<Program>) -> RaptorResult<Vec<Arc<Program>>> {
        let mut data: Vec<Arc<Program>> = vec![];
        let table = &mut data;

        self.recurse(program, &mut |prog| {
            table.push(prog);
        })?;

        Ok(data)
    }

    pub fn build(&mut self, program: Arc<Program>) -> RaptorResult<()> {
        let programs = self.stack(program)?;

        let mut layers: Vec<Utf8PathBuf> = vec!["layers/empty".into()];

        for prog in programs {
            let hash = Cacher::cache_key(&prog)?;

            let layer = Cacher::layer_info(&prog, hash);

            let layer_name = layer.name().to_string();
            let work_path = layer.work_path();
            let done_path = layer.done_path();

            if std::fs::exists(layer.done_path())? {
                info!("{} {}", "Completed".bright_white(), layer_name.yellow());
            } else {
                info!(
                    "{} {}: {}",
                    "Building".bright_white(),
                    layer_name.yellow(),
                    layer.work_path().green()
                );

                if self.dry_run {
                    let mut exec = PrintExecutor::new();
                    exec.run(&self.loader, &prog)?;
                } else {
                    let sandbox = Sandbox::new(&layers, Utf8Path::new(&layer.work_path()))?;

                    let mut exec = Executor::new(sandbox, layer);

                    exec.run(&self.loader, &prog)?;

                    exec.finish()?;
                    debug!("Layer {layer_name} finished. Moving {work_path} -> {done_path}");
                    std::fs::rename(&work_path, &done_path)?;
                }
            }

            layers.push(Utf8PathBuf::from(done_path));
        }

        Ok(())
    }
}
