use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{fs, thread};

use camino::Utf8PathBuf;
use camino_tempfile::Utf8TempDir;

use raptor::RaptorResult;
use raptor::build::{Cacher, RaptorBuilder};
use raptor::dsl::Program;
use raptor::program::Loader;
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
    builder: RaptorBuilder<'static>,
    tempdir: Utf8TempDir,
    hash: u64,
}

impl Tester {
    const PROGRAM_MODULE: &str = "program";
    const PROGRAM_NAME: &str = "program.rapt";

    fn setup(
        program: impl Writable,
        init: impl Fn(&Self) -> RaptorResult<()>,
    ) -> RaptorResult<Self> {
        let tempdir = Utf8TempDir::new()?;

        let loader = Loader::new()?.with_base(&tempdir);
        let builder = RaptorBuilder::new(loader, true);

        let mut res = Self {
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
    ) -> RaptorResult<()> {
        // clear cache to prevent stale programs
        self.builder.clear_cache();

        fun(self)?;

        let hash = self.program_hash()?;
        if self.hash == hash {
            eprintln!("{}", self.load(self.program_path().as_str())?);
            panic!("Program hash did not change after changing {act}");
        }
        self.hash = hash;
        Ok(())
    }

    /// Perform a (non-semantic) modification to one or more input files, and verify that the
    /// build hash does NOT change as a result.
    fn expect_same(
        &mut self,
        act: &str,
        fun: impl Fn(&mut Self) -> RaptorResult<()>,
    ) -> RaptorResult<()> {
        // clear cache to prevent stale programs
        self.builder.clear_cache();

        fun(self)?;

        let hash = self.program_hash()?;
        if self.hash != hash {
            eprintln!("{}", self.load(self.program_path().as_str())?);
            panic!("Program hash changed after changing {act}");
        }

        Ok(())
    }

    fn path(&self, name: &str) -> Utf8PathBuf {
        self.tempdir.path().join(name)
    }

    fn program_path(&self) -> Utf8PathBuf {
        self.path(Self::PROGRAM_NAME)
    }

    fn program_module() -> ModuleName {
        ModuleName::absolute(vec![Self::PROGRAM_MODULE.to_string()], None)
    }

    fn program_write(&self, value: impl Writable) -> RaptorResult<()> {
        Ok(fs::write(self.program_path(), value.into_string())?)
    }

    fn program_hash(&self) -> RaptorResult<u64> {
        self.hash(self.program_path().as_str())
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

    fn touch(&self, name: &str) -> RaptorResult<()> {
        // the hash depends on the mtime, so let enough time pass for the next
        // ctime to be different
        thread::sleep(Duration::from_millis(10));
        File::open(self.path(name))?.set_modified(SystemTime::now())?;

        Ok(())
    }

    fn load(&self, name: &str) -> RaptorResult<Arc<Program>> {
        self.builder.load(self.path(name))
    }

    fn hash(&self, name: &str) -> RaptorResult<u64> {
        let prog = self.load(name)?;
        Cacher::cache_key(&prog, &self.builder)
    }
}

#[test]
fn dep_copy() -> RaptorResult<()> {
    let mut test = Tester::setup(["COPY a a"], |test| test.write("a", "1234"))?;

    test.expect_new("COPY src", |test| test.touch("a"))?;

    Ok(())
}

#[test]
fn dep_render() -> RaptorResult<()> {
    let mut test = Tester::setup(["RENDER a a"], |test| test.write("a", "1234"))?;

    test.expect_new("RENDER src", |test| test.touch("a"))?;

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
    test.expect_new("FROM src 4", |test| test.append_inst(Tester::PROGRAM_NAME))?;

    test.expect_same("FROM src 1", |test| test.touch("c.rapt"))?;
    test.expect_same("FROM src 2", |test| test.touch("b.rapt"))?;
    test.expect_same("FROM src 3", |test| test.touch("a.rapt"))?;
    test.expect_same("FROM src 4", |test| test.touch(Tester::PROGRAM_NAME))?;

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
    test.expect_new("FROM src 3", |test| test.append_inst(Tester::PROGRAM_NAME))?;

    test.expect_new("FROM src 1", |test| test.touch("b.rinc"))?;
    test.expect_same("FROM src 2", |test| test.touch("a.rapt"))?;
    test.expect_same("FROM src 3", |test| test.touch(Tester::PROGRAM_NAME))?;

    Ok(())
}

#[test]
fn dep_self() -> RaptorResult<()> {
    let mut test = Tester::setup([""], |test| test.write("a.rapt", ""))?;

    test.expect_new("program", |test| test.append_inst(Tester::PROGRAM_NAME))?;

    test.expect_same("program", |test| test.touch(Tester::PROGRAM_NAME))?;

    Ok(())
}

#[test]
fn dep_include() -> RaptorResult<()> {
    let mut test = Tester::setup(["INCLUDE a"], |test| test.write("a.rinc", ""))?;

    test.expect_new("include file 1", |test| test.touch("a.rinc"))?;

    Ok(())
}

#[test]
fn dep_include2() -> RaptorResult<()> {
    let mut test = Tester::setup(["INCLUDE a"], |test| {
        test.write("a.rinc", "INCLUDE b")?;
        test.write("b.rinc", "")
    })?;

    test.expect_new("include file 1", |test| test.touch("a.rinc"))?;
    test.expect_new("include file 2", |test| test.touch("b.rinc"))?;

    Ok(())
}
