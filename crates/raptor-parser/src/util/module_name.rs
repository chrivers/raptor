use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModuleRoot {
    Relative,
    Absolute,
    Package(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModuleName {
    root: ModuleRoot,
    names: Vec<String>,
}

impl ModuleName {
    #[must_use]
    pub fn new(mut names: Vec<String>) -> Self {
        if names.first().is_some_and(|f| f.starts_with('$')) {
            let mut root = names.remove(0);
            root.remove(0);
            if root.is_empty() {
                Self::absolute(names)
            } else {
                Self::package(root, names)
            }
        } else {
            Self::relative(names)
        }
    }

    #[must_use]
    pub const fn build(root: ModuleRoot, names: Vec<String>) -> Self {
        Self { root, names }
    }

    #[must_use]
    pub const fn relative(names: Vec<String>) -> Self {
        Self {
            root: ModuleRoot::Relative,
            names,
        }
    }

    #[must_use]
    pub const fn absolute(names: Vec<String>) -> Self {
        Self {
            root: ModuleRoot::Absolute,
            names,
        }
    }

    #[must_use]
    pub const fn package(root: String, names: Vec<String>) -> Self {
        Self {
            root: ModuleRoot::Package(root),
            names,
        }
    }

    #[must_use]
    pub fn root(&self) -> Option<&str> {
        match &self.root {
            ModuleRoot::Relative | ModuleRoot::Absolute => None,
            ModuleRoot::Package(pkg) => Some(pkg),
        }
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

        /* assert_eq!(name.to_program_path(), "a/b.rapt"); */
        /* assert_eq!(name.to_include_path(), "a/b.rinc"); */
        assert_eq!(name.parts(), &["a", "b"]);
    }

    #[test]
    fn root() {
        let name = ModuleName::new(vec![
            String::from("$foo"),
            String::from("a"),
            String::from("b"),
        ]);

        assert_eq!(name.root(), Some("foo"));
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
