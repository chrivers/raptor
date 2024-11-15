use std::fmt::Debug;

#[derive(Clone)]
pub struct InstInvoke {
    pub args: Vec<String>,
}

impl Debug for InstInvoke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "INVOKE {}", self.args.join(" "))
    }
}
