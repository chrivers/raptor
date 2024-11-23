use std::fmt::Debug;

#[derive(Clone)]
pub struct InstWorkdir {
    pub dir: String,
}

impl Debug for InstWorkdir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WORKDIR {}", self.dir)
    }
}
