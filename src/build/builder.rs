use std::collections::HashMap;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use minijinja::context;

use crate::dsl::Program;
use crate::program::{Executor, Loader};
use crate::sandbox::Sandbox;
use crate::RaptorResult;

pub struct RaptorBuilder<'a> {
    loader: Loader<'a>,
    programs: HashMap<Utf8PathBuf, Arc<Program>>,
}

impl<'a> RaptorBuilder<'a> {
    pub fn new(loader: Loader<'a>) -> Self {
        Self {
            loader,
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

    pub fn exec(&self, program: &Program) -> RaptorResult<()> {
        let sandbox = Sandbox::new(&["layers/empty".into()], "layers/tmp".into())?;

        let mut exec = Executor::new(sandbox);

        exec.run(&self.loader, program)?;

        exec.finish()
    }
}
