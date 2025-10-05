use std::collections::{HashMap, HashSet};

use crate::RaptorResult;
use crate::build::BuildTarget;
use raptor_parser::ast::FromSource;

pub struct BuildTargetStats {
    pub targets: HashMap<String, BuildTarget>,
    pub roots: HashSet<String>,
    pub map: HashMap<String, String>,
    pub rmap: HashMap<String, HashSet<String>>,
}

impl Default for BuildTargetStats {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildTargetStats {
    #[must_use]
    pub fn new() -> Self {
        Self {
            targets: HashMap::new(),
            roots: HashSet::new(),
            map: HashMap::new(),
            rmap: HashMap::new(),
        }
    }

    pub fn merge(&mut self, stack: Vec<BuildTarget>) -> RaptorResult<()> {
        for layer in stack {
            match layer {
                BuildTarget::Program(ref program) => {
                    let name = program.path.file_stem().unwrap().to_string();

                    if let Some((from, _origin)) = program.from() {
                        let key = match &from {
                            FromSource::Raptor(from) => from.to_string(),
                            FromSource::Docker(image) => {
                                if image.contains('/') {
                                    format!("docker://{image}")
                                } else {
                                    format!("docker://library/{image}")
                                }
                            }
                        };

                        self.rmap
                            .entry(key.clone())
                            .or_default()
                            .insert(name.clone());
                        self.map.insert(name.clone(), key);
                    } else {
                        self.roots.insert(name.clone());
                    }
                    self.targets.insert(name, layer);
                }
                BuildTarget::DockerSource(ref src) => {
                    let name = format!("docker://{src}");
                    self.targets.insert(name.clone(), layer);
                    self.roots.insert(name);
                }
            }
        }

        Ok(())
    }
}
