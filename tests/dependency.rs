use std::time::Duration;
use std::{fs, thread};

use camino_tempfile::Utf8TempDir;

use raptor::build::{Cacher, RaptorBuilder};
use raptor::program::Loader;
use raptor::RaptorResult;

fn setup() -> RaptorResult<(RaptorBuilder<'static>, Utf8TempDir)> {
    let tempdir = Utf8TempDir::new()?;

    let loader = Loader::new()?.with_base(&tempdir);
    let builder = RaptorBuilder::new(loader, true);

    Ok((builder, tempdir))
}

#[test]
fn dep_copy() -> RaptorResult<()> {
    let (mut builder, tempdir) = setup()?;

    let path = tempdir.path();
    let path_rapt = path.join("test.rapt");
    let path_test = path.join("a");

    fs::write(&path_rapt, "COPY a a\n")?;

    fs::write(&path_test, "1234")?;
    let prog = builder.load(&path_rapt)?;
    let hash_a = Cacher::cache_key(&prog)?;

    // the hash depends on the mtime, so let enough time pass for the next mtime
    // to be different
    thread::sleep(Duration::from_millis(10));

    fs::write(&path_test, "ABCD")?;
    let prog = builder.load(&path_rapt)?;
    let hash_b = Cacher::cache_key(&prog)?;

    println!("{hash_a:16X} vs {hash_b:16X}");
    assert_ne!(hash_a, hash_b);
    Ok(())
}
