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

trait ShellRun {
    fn shell(&mut self, cmd: &[&str]) -> RaptorResult<()>;
}

impl ShellRun for Sandbox {
    fn shell(&mut self, cmd: &[&str]) -> RaptorResult<()> {
        let mut args = vec!["/bin/sh", "-c"];
        args.extend(cmd);
        self.run(
            &args
                .into_iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        )
    }
}

#[test]
fn nspawn_basic() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("basic")?;
    assert_ne!(sbx.get_mount_dir(), None);
    assert_ne!(sbx.get_temp_dir(), None);
    sbx.close()?;
    assert_eq!(sbx.get_mount_dir(), None);
    assert_eq!(sbx.get_temp_dir(), None);
    Ok(())
}

#[test]
fn nspawn_drop() -> RaptorResult<()> {
    let sbx = spawn_sandbox("drop")?;
    let mount_path = sbx.get_mount_dir().unwrap().to_owned();
    assert!(mount_path.exists());
    drop(sbx);
    assert!(!mount_path.exists());
    Ok(())
}

#[test]
fn nspawn_exit_status() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("exit_status")?;

    sbx.shell(&["true"])?;

    assert!(matches!(
        sbx.shell(&["false"]).unwrap_err(),
        RaptorError::SandboxRunError(es) if es.code() == Some(1)
    ));
    assert!(matches!(
        sbx.shell(&["exit 100"]).unwrap_err(),
        RaptorError::SandboxRunError(es) if es.code() == Some(100)
    ));
    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_workdir() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("workdir")?;
    sbx.shell(&["[ $PWD == / ]"])?;
    sbx.chdir("/bin")?;
    sbx.shell(&["[ $PWD == /bin ]"])?;
    sbx.chdir("../usr")?;
    sbx.shell(&["[ $PWD == /usr ]"])?;
    sbx.chdir("..")?;
    sbx.shell(&["[ $PWD == / ]"])?;
    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_setenv() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("setenv")?;
    sbx.shell(&["[ x${FOO} == x ]"])?;
    sbx.setenv("FOO", "BAR")?;
    sbx.shell(&["[ x${FOO} == xBAR ]"])?;
    sbx.setenv("FOO", "OTHER")?;
    sbx.shell(&["[ x${FOO} == xOTHER ]"])?;
    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_write_data() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("write")?;
    let mut fd = sbx.create_file_handle("/tmp/a".into(), None, None)?;
    fd.write_all(b"Hello world\n")?;
    drop(fd);

    fd = sbx.create_file_handle("/tmp/b".into(), None, None)?;
    fd.write_all("f0ef7081e1539ac00ef5b761b4fb01b3  a\n".as_bytes())?;
    drop(fd);

    sbx.shell(&["cd /tmp && md5sum -cs b"])?;

    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_write_opts() -> RaptorResult<()> {
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

    sbx.shell(&["[ $(stat -c %u /tmp/c) -eq 0 ]"])?;

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

    sbx.shell(&["[ $(stat -c %u /tmp/c) -eq 1000 ]"])?;

    sbx.close()?;
    Ok(())
}
