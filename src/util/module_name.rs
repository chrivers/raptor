use std::fmt::Display;

use camino::Utf8PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModuleName {
    names: Vec<String>,
}

impl ModuleName {
    #[must_use]
    pub const fn new(names: Vec<String>) -> Self {
        Self { names }
    }

    #[must_use]
    pub fn to_program_path(&self) -> Utf8PathBuf {
        format!("{}.rapt", self.names.join("/")).into()
    }

    #[must_use]
    pub fn to_include_path(&self) -> Utf8PathBuf {
        format!("{}.rinc", self.names.join("/")).into()
    }

    #[must_use]
    pub fn parts(&self) -> &[String] {
        &self.names
    }
}

impl Display for ModuleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.names.join("."))
    }
}

impl From<&str> for ModuleName {
    fn from(value: &str) -> Self {
        if value.is_empty() {
            return Self::new(vec![]);
        }

        Self::new(value.split('.').map(ToString::to_string).collect())
    }
}
