use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use std::time::SystemTime;

use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::Utf8TempDir;

use raptor::RaptorResult;
use raptor::build::{Cacher, RaptorBuilder};
use raptor::dsl::Program;
use raptor::program::Loader;
use raptor::sandbox::Sandbox;
use raptor_parser::ast::Origin;
use raptor_parser::util::module_name::ModuleName;

trait Writable {
    fn into_string(self) -> String;
}

impl Writable for &str {
    fn into_string(self) -> String {
        self.to_string()
    }
}

impl<const N: usize> Writable for [&str; N] {
    fn into_string(self) -> String {
        self[..].into_string()
    }
}

impl Writable for &[&str] {
    fn into_string(self) -> String {
        let mut res = self.join("\n");
        res.push('\n');
        res
    }
}

struct Tester {
    program_name: ModuleName,
    builder: RaptorBuilder<'static>,
    tempdir: Utf8TempDir,
    hash: u64,
}

impl Tester {
    fn setup(
        program: impl Writable,
        init: impl Fn(&Self) -> RaptorResult<()>,
    ) -> RaptorResult<Self> {
        let name = ModuleName::from("$.program");
        Self::setup_custom(name, program, init)
    }

    fn setup_custom(
        program_name: ModuleName,
        program: impl Writable,
        init: impl Fn(&Self) -> RaptorResult<()>,
    ) -> RaptorResult<Self> {
        let tempdir = Utf8TempDir::new()?;
        let loader = Loader::new()?.with_base(&tempdir);
        let builder = RaptorBuilder::new(loader, Sandbox::find_falcon_dev().unwrap(), true);

        let mut res = Self {
            program_name,
            builder,
            tempdir,
            hash: 0,
        };
        res.program_write(program)?;
        init(&res)?;
        res.hash = res.program_hash()?;

        Ok(res)
    }

    /// Perform a modification to one or more input files, and verify that the
    /// build hash changes as a result.
    fn expect_new(
        &mut self,
        act: &str,
        fun: impl Fn(&mut Self) -> RaptorResult<()>,
    ) -> RaptorResult<u64> {
        // clear cache to prevent stale programs
        self.builder.clear_cache();

        fun(self)?;

        let hash = self.program_hash()?;
        if self.hash == hash {
            eprintln!("{}", self.load(&self.program_name)?);
            panic!("Program hash did not change after changing {act}");
        }
        self.hash = hash;

        Ok(hash)
    }

    /// Perform a (non-semantic) modification to one or more input files, and verify that the
    /// build hash does NOT change as a result.
    fn expect_same(
        &mut self,
        act: &str,
        fun: impl Fn(&mut Self) -> RaptorResult<()>,
    ) -> RaptorResult<u64> {
        // clear cache to prevent stale programs
        self.builder.clear_cache();

        fun(self)?;

        let hash = self.program_hash()?;
        if self.hash != hash {
            eprintln!("{}", self.load(&self.program_name)?);
            panic!("Program hash changed after changing {act}");
        }

        Ok(hash)
    }

    /// Perform a modification to one or more input files, and verify that the
    /// build hash becomes the expected value
    fn expect_hash(
        &mut self,
        act: &str,
        fun: impl Fn(&mut Self) -> RaptorResult<()>,
        expected: u64,
    ) -> RaptorResult<u64> {
        // clear cache to prevent stale programs
        self.builder.clear_cache();

        fun(self)?;

        let hash = self.program_hash()?;
        if hash != expected {
            eprintln!("{}", self.load(&self.program_name)?);
            panic!("Program hash did not change after changing {act}");
        }
        self.hash = expected;

        Ok(hash)
    }

    fn path(&self, name: impl AsRef<Utf8Path>) -> Utf8PathBuf {
        self.tempdir.path().join(name)
    }

    fn program_path(&self) -> String {
        let program = self
            .builder
            .loader()
            .to_program_path(&self.program_name, &Origin::inline())
            .unwrap();

        self.path(program).to_string()
    }

    fn program_write(&self, value: impl Writable) -> RaptorResult<()> {
        Ok(fs::write(self.program_path(), value.into_string())?)
    }

    fn program_hash(&self) -> RaptorResult<u64> {
        self.hash(&self.program_name)
    }

    fn write(&self, name: &str, value: impl Writable) -> RaptorResult<()> {
        Ok(fs::write(self.path(name), value.into_string())?)
    }

    fn append(&self, name: &str, value: impl Writable) -> RaptorResult<()> {
        let val = value.into_string();

        OpenOptions::new()
            .append(true)
            .open(self.path(name))?
            .write_all(val.as_bytes())?;

        Ok(())
    }

    fn append_inst(&self, name: &str) -> RaptorResult<()> {
        self.append(name, ["", &format!("RUN echo {name}")])
    }

    fn mkdir(&self, name: &str) -> RaptorResult<()> {
        Ok(fs::create_dir_all(self.path(name))?)
    }

    fn touch(&self, name: &str) -> RaptorResult<()> {
        File::open(self.path(name))?.set_modified(SystemTime::now())?;

        Ok(())
    }

    fn load(&self, name: &ModuleName) -> RaptorResult<Arc<Program>> {
        self.builder.load(name)
    }

    fn hash(&self, name: &ModuleName) -> RaptorResult<u64> {
        let prog = self.load(name)?;
        Cacher::cache_key(&prog, &self.builder)
    }
}

#[test]
fn dep_copy() -> RaptorResult<()> {
    let mut test = Tester::setup(["COPY a a"], |test| test.write("a", "1234"))?;

    test.expect_same("COPY src", |test| test.touch("a"))?;

    Ok(())
}

#[test]
fn dep_render() -> RaptorResult<()> {
    let mut test = Tester::setup(["RENDER a a"], |test| test.write("a", "1234"))?;

    test.expect_same("RENDER src", |test| test.touch("a"))?;
    test.expect_new("RENDER src", |test| test.append("a", "more"))?;

    Ok(())
}

#[test]
fn dep_from() -> RaptorResult<()> {
    let mut test = Tester::setup(["FROM a"], |test| test.write("a.rapt", ""))?;

    test.expect_new("FROM src", |test| test.append_inst("a.rapt"))?;

    test.expect_same("FROM src", |test| test.touch("a.rapt"))?;

    Ok(())
}

#[test]
fn dep_from2() -> RaptorResult<()> {
    let mut test = Tester::setup(["FROM a"], |test| {
        test.write("a.rapt", "FROM b")?;
        test.write("b.rapt", "FROM c")?;
        test.write("c.rapt", "")
    })?;

    test.expect_new("FROM src 1", |test| test.append_inst("c.rapt"))?;
    test.expect_new("FROM src 2", |test| test.append_inst("b.rapt"))?;
    test.expect_new("FROM src 3", |test| test.append_inst("a.rapt"))?;
    test.expect_new("FROM src 4", |test| test.append_inst(&test.program_path()))?;

    test.expect_same("FROM src 1", |test| test.touch("c.rapt"))?;
    test.expect_same("FROM src 2", |test| test.touch("b.rapt"))?;
    test.expect_same("FROM src 3", |test| test.touch("a.rapt"))?;
    test.expect_same("FROM src 4", |test| test.touch(&test.program_path()))?;

    Ok(())
}

#[test]
fn dep_from3() -> RaptorResult<()> {
    let mut test = Tester::setup(["FROM a"], |test| {
        test.write("a.rapt", "INCLUDE b")?;
        test.write("b.rinc", "")
    })?;

    test.expect_new("FROM src 1", |test| test.append_inst("b.rinc"))?;
    test.expect_new("FROM src 2", |test| test.append_inst("a.rapt"))?;
    test.expect_new("FROM src 3", |test| test.append_inst(&test.program_path()))?;

    test.expect_same("FROM src 1", |test| test.touch("b.rinc"))?;
    test.expect_same("FROM src 2", |test| test.touch("a.rapt"))?;
    test.expect_same("FROM src 3", |test| test.touch(&test.program_path()))?;

    Ok(())
}

#[test]
fn dep_self() -> RaptorResult<()> {
    let mut test = Tester::setup([""], |test| test.write("a.rapt", ""))?;

    test.expect_new("program", |test| test.append_inst(&test.program_path()))?;

    test.expect_same("program", |test| test.touch(&test.program_path()))?;

    Ok(())
}

#[test]
fn dep_include() -> RaptorResult<()> {
    let mut test = Tester::setup(["INCLUDE a"], |test| test.write("a.rinc", ""))?;

    test.expect_same("include file 1", |test| test.touch("a.rinc"))?;

    Ok(())
}

#[test]
fn dep_include2() -> RaptorResult<()> {
    let mut test = Tester::setup(["INCLUDE a"], |test| {
        test.write("a.rinc", "INCLUDE b")?;
        test.write("b.rinc", "")
    })?;

    test.expect_same("include file 1", |test| test.touch("a.rinc"))?;
    test.expect_same("include file 2", |test| test.touch("b.rinc"))?;

    test.expect_new("include file 1", |test| test.append_inst("a.rinc"))?;
    test.expect_new("include file 2", |test| test.append_inst("b.rinc"))?;

    Ok(())
}

#[test]
fn dep_include3() -> RaptorResult<()> {
    let mut test = Tester::setup(["INCLUDE inc.a"], |test| {
        test.mkdir("inc")?;
        test.write("inc/src", "data")?;
        test.write("inc/rnd", "data")?;
        test.write("inc/a.rinc", "INCLUDE b")?;
        test.write("inc/b.rinc", ["COPY src /dest1", "RENDER rnd /dest2"])?;
        Ok(())
    })?;

    test.expect_same("include file 1", |test| test.touch("inc/a.rinc"))?;
    test.expect_same("include file 2", |test| test.touch("inc/b.rinc"))?;

    test.expect_new("include file 1", |test| test.append_inst("inc/a.rinc"))?;
    test.expect_new("include file 2", |test| test.append_inst("inc/b.rinc"))?;

    Ok(())
}

#[test]
fn dep_include4() -> RaptorResult<()> {
    let mut test = Tester::setup(["INCLUDE inc.a"], |test| {
        test.mkdir("inc")?;
        test.write("inc/a.rinc", "RUN echo foo")?;
        test.write("inc/b.rinc", "RUN echo foo")?;
        Ok(())
    })?;

    test.expect_same("include file 1", |test| test.program_write("INCLUDE inc.b"))?;
    test.expect_same("include file a", |test| {
        test.write("inc/a.rinc", "RUN echo bar")
    })?;
    test.expect_new("include file b", |test| {
        test.write("inc/b.rinc", "RUN echo bar")
    })?;

    Ok(())
}

#[test]
fn dep_instance() -> RaptorResult<()> {
    let mut test = Tester::setup(["INCLUDE a@one"], |test| {
        test.write("a@.rinc", "WRITE {{instance[0]}} /tmp/foo")
    })?;

    test.expect_new("instance", |test| test.program_write("INCLUDE a@two"))?;

    Ok(())
}

#[test]
fn dep_instance2() -> RaptorResult<()> {
    let mut test = Tester::setup(["INCLUDE a"], |test| {
        test.write("a.rinc", "INCLUDE b@one")?;
        test.write("b@.rinc", "WRITE {{instance}} /tmp/foo")
    })?;

    let orig = test.hash;

    test.expect_new("instance", |test| test.write("a.rinc", "INCLUDE b@two"))?;
    test.expect_hash(
        "instance",
        |test| test.write("a.rinc", "INCLUDE b@one"),
        orig,
    )?;

    Ok(())
}

#[test]
fn dep_instance3() -> RaptorResult<()> {
    let mut test = Tester::setup_custom(
        ModuleName::from("$.program@one"),
        ["WRITE {{instance}} /tmp/foo"],
        |_test| Ok(()),
    )?;

    let orig = test.hash;

    test.program_name = ModuleName::from("$.program@two");
    test.builder.clear_cache();
    assert_ne!(orig, test.program_hash()?);

    test.program_name = ModuleName::from("$.program@one");
    test.builder.clear_cache();
    assert_eq!(orig, test.program_hash()?);

    Ok(())
}
