use std::fmt::Debug;

#[derive(Clone)]
pub struct InstFrom {
    pub from: String,
}

impl Debug for InstFrom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FROM {}", self.from)
    }
}
