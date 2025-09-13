use std::fs::File;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{fs, thread};

use camino::Utf8PathBuf;
use camino_tempfile::Utf8TempDir;

use raptor::build::{Cacher, RaptorBuilder};
use raptor::dsl::Program;
use raptor::program::Loader;
use raptor::RaptorResult;

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

    fn step(&mut self, act: &str, fun: impl Fn(&mut Self) -> RaptorResult<()>) -> RaptorResult<()> {
        fun(self)?;

        let hash = self.program_hash()?;
        if self.hash == hash {
            eprintln!("{}", self.load(self.program_path().as_str())?);
            panic!("Program hash did not change after {act}");
        }
        self.hash = hash;
        Ok(())
    }

    fn path(&self, name: &str) -> Utf8PathBuf {
        self.tempdir.path().join(name)
    }

    fn program_path(&self) -> Utf8PathBuf {
        self.path(Self::PROGRAM_NAME)
    }

    fn program_write(&self, value: impl Writable) -> RaptorResult<()> {
        Ok(fs::write(self.program_path(), value.into_string())?)
    }

    fn program_hash(&mut self) -> RaptorResult<u64> {
        self.hash(self.program_path().as_str())
    }

    fn write(&self, name: &str, value: impl Writable) -> RaptorResult<()> {
        Ok(fs::write(self.path(name), value.into_string())?)
    }

    fn touch(&self, name: &str) -> RaptorResult<()> {
        // the hash depends on the mtime, so let enough time pass for the next
        // ctime to be different
        thread::sleep(Duration::from_millis(10));
        File::open(self.path(name))?.set_modified(SystemTime::now())?;

        Ok(())
    }

    fn load(&mut self, name: &str) -> RaptorResult<Arc<Program>> {
        self.builder.load(self.path(name))
    }

    fn hash(&mut self, name: &str) -> RaptorResult<u64> {
        let prog = self.load(name)?;
        Cacher::cache_key(&prog, &mut self.builder)
    }
}

#[test]
fn dep_copy() -> RaptorResult<()> {
    let mut test = Tester::setup(["COPY a a"], |test| test.write("a", "1234"))?;

    test.step("changing COPY src", |test| test.touch("a"))?;

    Ok(())
}

#[test]
fn dep_render() -> RaptorResult<()> {
    let mut test = Tester::setup(["RENDER a a"], |test| test.write("a", "1234"))?;

    test.step("changing RENDER src", |test| test.touch("a"))?;

    Ok(())
}

#[test]
fn dep_self() -> RaptorResult<()> {
    let mut test = Tester::setup([""], |test| test.write("a.rapt", ""))?;

    test.step("changing program file", |test| {
        test.touch(Tester::PROGRAM_NAME)
    })?;

    Ok(())
}

#[test]
fn dep_include() -> RaptorResult<()> {
    let mut test = Tester::setup(["INCLUDE \"a.rinc\""], |test| test.write("a.rinc", ""))?;

    test.step("changing include file", |test| test.touch("a.rinc"))?;

    Ok(())
}
