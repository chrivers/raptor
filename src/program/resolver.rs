use camino::{Utf8Path, Utf8PathBuf};
use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use raptor_parser::ast::Origin;
use raptor_parser::util::module_name::{ModuleName, ModuleRoot};

use crate::{RaptorError, RaptorResult};

pub struct Resolver {
    base: Utf8PathBuf,
    packages: DashMap<String, Utf8PathBuf>,
}

impl Resolver {
    #[must_use]
    pub fn new(base: Utf8PathBuf) -> Self {
        Self {
            base,
            packages: DashMap::new(),
        }
    }

    #[must_use]
    pub fn base(&self) -> &Utf8Path {
        &self.base
    }

    pub fn set_base(&mut self, base: impl AsRef<Utf8Path>) {
        self.base = base.as_ref().to_path_buf();
    }

    pub fn add_package(&self, name: String, path: Utf8PathBuf) {
        self.packages.insert(name, path);
    }

    #[must_use]
    pub fn get_package(&self, name: &str) -> Option<Ref<'_, String, Utf8PathBuf>> {
        self.packages.get(name)
    }

    #[must_use]
    pub fn path(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
        self.base.join(path)
    }

    pub fn to_path(
        &self,
        root: &ModuleRoot,
        origin: &Origin,
        end: &Utf8Path,
    ) -> RaptorResult<Utf8PathBuf> {
        let res = match root {
            ModuleRoot::Relative => origin.path_for(end)?,
            ModuleRoot::Absolute => end.into(),
            ModuleRoot::Package(pkg) => {
                let package = self
                    .get_package(pkg)
                    .ok_or_else(|| RaptorError::PackageNotFound(pkg.clone(), origin.clone()))?;
                package.join(end)
            }
        };

        Ok(res)
    }

    pub fn to_program_path(&self, name: &ModuleName, origin: &Origin) -> RaptorResult<Utf8PathBuf> {
        let mut end = Utf8PathBuf::new();
        end.extend(name.parts());
        end.set_extension("rapt");
        self.to_path(name.root(), origin, &end)
    }

    pub fn to_include_path(&self, name: &ModuleName, origin: &Origin) -> RaptorResult<Utf8PathBuf> {
        let mut end = Utf8PathBuf::new();
        end.extend(name.parts());
        end.set_extension("rinc");
        self.to_path(name.root(), origin, &end)
    }

    pub fn resolve_logical_path(&self, path: impl AsRef<Utf8Path>) -> RaptorResult<Utf8PathBuf> {
        let mut comps = vec![];
        for comp in path.as_ref().components() {
            let name = comp.as_str();
            if let Some(suffix) = name.strip_prefix('$') {
                if let Some(link) = self.get_package(suffix) {
                    comps.push(link.to_string());
                } else {
                    return Err(RaptorError::MissingLink(name.to_string()));
                }
            } else {
                comps.push(name.to_string());
            }
        }

        Ok(Utf8PathBuf::from_iter(comps))
    }
}

#[cfg(test)]
mod tests {
    use camino::{Utf8Path, Utf8PathBuf};
    use raptor_parser::ast::Origin;
    use raptor_parser::util::module_name::ModuleRoot;

    use crate::RaptorResult;
    use crate::program::Resolver;

    macro_rules! path_test {
        ($resolver:expr, $origin:expr, $name:expr, [$a:expr, $b:expr, $c:expr]) => {
            let rel = &ModuleRoot::Relative;
            let abs = &ModuleRoot::Absolute;
            let pkg = &ModuleRoot::Package(String::from("pkg"));
            assert_eq!($resolver.to_path(rel, &$origin, &$name)?, $a);
            assert_eq!($resolver.to_path(abs, &$origin, &$name)?, $b);
            assert_eq!($resolver.to_path(pkg, &$origin, &$name)?, $c);
        };
    }

    #[test]
    fn resolve_inline() -> RaptorResult<()> {
        let mut resolv = Resolver::new(Utf8PathBuf::new());
        resolv.add_package("pkg".into(), "pkgpath".into());

        let inline = Origin::inline();
        let name = Utf8Path::new("name");

        path_test!(resolv, inline, name, ["name", "name", "pkgpath/name"]);
        resolv.set_base("base");
        path_test!(resolv, inline, name, ["name", "name", "pkgpath/name"]);

        Ok(())
    }

    #[test]
    fn resolve_base() -> RaptorResult<()> {
        let mut resolv = Resolver::new(Utf8PathBuf::new());
        resolv.add_package("pkg".into(), "pkgpath".into());

        let inline = Origin::make("base/foo.rapt", 0..0);
        let name = Utf8Path::new("name");

        path_test!(resolv, inline, name, ["base/name", "name", "pkgpath/name"]);
        resolv.set_base("base");
        path_test!(resolv, inline, name, ["base/name", "name", "pkgpath/name"]);

        Ok(())
    }
}
