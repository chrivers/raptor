use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq)]
pub struct InstEnvAssign {
    pub key: String,
    pub value: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct InstEnv {
    pub env: Vec<InstEnvAssign>,
}

impl Debug for InstEnvAssign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

impl Debug for InstEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ENV")?;
        for e in &self.env {
            write!(f, " {e:?}")?;
        }
        Ok(())
    }
}
