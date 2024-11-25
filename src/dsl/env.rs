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

impl InstEnvAssign {
    pub fn new(key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        Self {
            key: key.as_ref().to_string(),
            value: value.as_ref().to_string(),
        }
    }
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
