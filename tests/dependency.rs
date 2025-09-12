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

struct Tester {
    builder: RaptorBuilder<'static>,
    tempdir: Utf8TempDir,
}

trait Writable {
    fn as_string(self) -> String;
}

impl Writable for &str {
    fn as_string(self) -> String {
        self.to_string()
    }
}

impl<const N: usize> Writable for [&str; N] {
    fn as_string(self) -> String {
        self[..].as_string()
    }
}

impl Writable for &[&str] {
    fn as_string(self) -> String {
        let mut res = self.join("\n");
        res.push('\n');
        res
    }
}

impl Tester {
    fn setup() -> RaptorResult<Self> {
        let tempdir = Utf8TempDir::new()?;

        let loader = Loader::new()?.with_base(&tempdir);
        let builder = RaptorBuilder::new(loader, true);

        Ok(Self { builder, tempdir })
    }

    fn path(&self, name: &str) -> Utf8PathBuf {
        self.tempdir.path().join(name)
    }

    fn write(&self, name: &str, value: impl Writable) -> RaptorResult<()> {
        Ok(fs::write(self.path(name), value.as_string())?)
    }

    fn touch(&self, name: &str) -> RaptorResult<()> {
        // the hash depends on the mtime, so let enough time pass for the next
        // ctime to be different
        thread::sleep(Duration::from_millis(10));
        File::open(&self.path(name))?.set_modified(SystemTime::now())?;

        Ok(())
    }

    fn load(&mut self, name: &str) -> RaptorResult<Arc<Program>> {
        self.builder.load(&self.path(name))
    }

    fn hash(&mut self, name: &str) -> RaptorResult<u64> {
        let prog = self.load(name)?;
        Cacher::cache_key(&prog)
    }
}

#[test]
fn dep_copy() -> RaptorResult<()> {
    let mut test = Tester::setup()?;

    test.write("test.rapt", ["COPY a a"])?;
    test.write("a", "1234")?;

    let hash_a = test.hash("test.rapt")?;
    test.touch("a")?;
    let hash_b = test.hash("test.rapt")?;

    println!("{hash_a:16X} vs {hash_b:16X}");
    assert_ne!(hash_a, hash_b);
    Ok(())
}
