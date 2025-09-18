use std::fmt::Display;

use crate::print::Theme;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstEnvAssign {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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

impl Display for InstEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.keyword("ENV")?;
        for env in &self.env {
            f.env_arg(env)?;
        }
        Ok(())
    }
}
