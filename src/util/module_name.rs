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

#[cfg(test)]
mod tests {
    use crate::util::module_name::ModuleName;

    #[test]
    fn basic() {
        let name = ModuleName::new(vec![String::from("a"), String::from("b")]);

        assert_eq!(name.to_program_path(), "a/b.rapt");
        assert_eq!(name.to_include_path(), "a/b.rinc");
        assert_eq!(name.parts(), &["a", "b"]);
    }

    #[test]
    fn format() {
        let name = ModuleName::new(vec![String::from("a"), String::from("b")]);

        assert_eq!(format!("{name}"), "a.b");
    }

    #[test]
    fn from0() {
        let name = ModuleName::from("");
        let expected: &[&str] = &[];

        assert_eq!(name.parts(), expected);
    }

    #[test]
    fn from1() {
        let name = ModuleName::from("a");
        let expected: &[&str] = &["a"];

        assert_eq!(name.parts(), expected);
    }

    #[test]
    fn from3() {
        let name = ModuleName::from("a.b.c");
        let expected: &[&str] = &["a", "b", "c"];

        assert_eq!(name.parts(), expected);
    }
}
