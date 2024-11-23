use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq)]
pub struct InstRun {
    pub run: Vec<String>,
}

impl Debug for InstRun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RUN {}", self.run.join(" "))
    }
}
