use std::io::Write;

use camino::Utf8Path;
use raptor::dsl::Chown;
use raptor::sandbox::Sandbox;
use raptor::{RaptorError, RaptorResult};

fn spawn_sandbox(name: &str) -> RaptorResult<Sandbox> {
    Sandbox::new(
        &[Utf8Path::new("tests/data/busybox")],
        &Utf8Path::new("tests/data/tmp").join(name),
    )
}

fn run(sbx: &mut Sandbox, cmd: &[&str]) -> RaptorResult<()> {
    let mut args = vec!["/bin/sh", "-c"];
    args.extend(cmd);
    sbx.run(
        &args
            .into_iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    )
}

#[test]
fn test_nspawn_basic() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("basic")?;
    sbx.close()?;
    Ok(())
}

#[test]
fn test_nspawn_run() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("run")?;
    run(&mut sbx, &["true"])?;
    assert!(matches!(
            run(&mut sbx, &["false"]).unwrap_err(),
            RaptorError::SandboxRunError(es) if es.code() == Some(1)
    ));
    assert!(matches!(
            run(&mut sbx, &["exit 100"]).unwrap_err(),
            RaptorError::SandboxRunError(es) if es.code() == Some(100)
    ));
    sbx.close()?;
    Ok(())
}

#[test]
fn test_nspawn_workdir() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("workdir")?;
    run(&mut sbx, &["[ $PWD == / ]"])?;
    sbx.chdir("/bin")?;
    run(&mut sbx, &["[ $PWD == /bin ]"])?;
    sbx.chdir("../usr")?;
    run(&mut sbx, &["[ $PWD == /usr ]"])?;
    sbx.chdir("..")?;
    run(&mut sbx, &["[ $PWD == / ]"])?;
    sbx.close()?;
    Ok(())
}

#[test]
fn test_nspawn_setenv() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("setenv")?;
    run(&mut sbx, &["[ x${FOO} == x ]"])?;
    sbx.setenv("FOO", "BAR")?;
    run(&mut sbx, &["[ x${FOO} == xBAR ]"])?;
    sbx.setenv("FOO", "OTHER")?;
    run(&mut sbx, &["[ x${FOO} == xOTHER ]"])?;
    sbx.close()?;
    Ok(())
}

#[test]
fn test_nspawn_write() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("write")?;
    let mut fd = sbx.create_file_handle("/tmp/a".into(), None, None)?;
    fd.write_all(b"Hello world\n")?;
    drop(fd);

    fd = sbx.create_file_handle("/tmp/b".into(), None, None)?;
    fd.write_all("f0ef7081e1539ac00ef5b761b4fb01b3  a\n".as_bytes())?;
    drop(fd);

    run(&mut sbx, &["cd /tmp && md5sum -cs b"])?;

    sbx.close()?;
    Ok(())
}

#[test]
fn test_nspawn_write_opts() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("write_opts")?;

    let mut fd = sbx.create_file_handle("/etc/passwd".into(), None, None)?;
    fd.write_all(b"root:x:0:0:root:/root:/bin/sh\n")?;
    fd.write_all(b"user:x:1000:1000:user:/home/user:/bin/sh\n")?;
    drop(fd);

    let mut fd = sbx.create_file_handle(
        "/tmp/c".into(),
        Some(Chown {
            user: Some("root".into()),
            group: None,
        }),
        None,
    )?;
    fd.write_all(b"Hello world\n")?;
    drop(fd);

    run(&mut sbx, &["[ $(stat -c %u /tmp/c) -eq 0 ]"])?;

    let mut fd = sbx.create_file_handle(
        "/tmp/c".into(),
        Some(Chown {
            user: Some("user".into()),
            group: None,
        }),
        None,
    )?;
    fd.write_all(b"Hello world\n")?;
    drop(fd);

    run(&mut sbx, &["[ $(stat -c %u /tmp/c) -eq 1000 ]"])?;

    sbx.close()?;
    Ok(())
}
