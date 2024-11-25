use camino::Utf8Path;
use raptor::dsl::Chown;
use raptor::sandbox::{Sandbox, SandboxExt};
use raptor::{RaptorError, RaptorResult};

fn spawn_sandbox(name: &str) -> RaptorResult<Sandbox> {
    Sandbox::new(
        &[Utf8Path::new("tests/data/busybox")],
        &Utf8Path::new("tests/data/tmp").join(name),
    )
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
    let mut sbx = spawn_sandbox("write_data")?;
    sbx.write_file("/tmp/a", None, None, b"Hello world\n")?;
    sbx.write_file(
        "/tmp/b",
        None,
        None,
        "f0ef7081e1539ac00ef5b761b4fb01b3  a\n",
    )?;

    sbx.shell(&["cd /tmp && md5sum -cs b"])?;

    sbx.close()?;
    Ok(())
}

fn write_etc_passwd(sbx: &mut Sandbox) -> RaptorResult<()> {
    sbx.write_file(
        "/etc/passwd",
        None,
        None,
        concat!(
            "root:x:0:0:root:/root:/bin/sh\n",
            "user:x:1000:1000:user:/home/user:/bin/sh\n"
        ),
    )
}

fn write_etc_group(sbx: &mut Sandbox) -> RaptorResult<()> {
    sbx.write_file(
        "/etc/group",
        None,
        None,
        concat!("root:x:0:\n", "user:x:1000:\n"),
    )
}

#[test]
fn nspawn_write_chown_user() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("write_chown_user")?;

    write_etc_passwd(&mut sbx)?;

    sbx.write_file("/tmp/c", Some(Chown::user("root")), None, b"Hello world\n")?;
    sbx.shell(&["[ $(stat -c %u /tmp/c) -eq 0 ]"])?;
    sbx.shell(&["[ $(stat -c %g /tmp/c) -eq 0 ]"])?;

    sbx.write_file("/tmp/c", Some(Chown::user("user")), None, b"Hello world\n")?;
    sbx.shell(&["[ $(stat -c %u /tmp/c) -eq 1000 ]"])?;
    sbx.shell(&["[ $(stat -c %g /tmp/c) -eq 0 ]"])?;

    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_write_chown_group() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("write_chown_group")?;

    write_etc_passwd(&mut sbx)?;
    write_etc_group(&mut sbx)?;

    sbx.write_file("/tmp/c", Some(Chown::group("root")), None, b"Hello world\n")?;
    sbx.shell(&["[ $(stat -c %u /tmp/c) -eq 0 ]"])?;
    sbx.shell(&["[ $(stat -c %g /tmp/c) -eq 0 ]"])?;

    sbx.write_file("/tmp/c", Some(Chown::group("user")), None, b"Hello world\n")?;
    sbx.shell(&["[ $(stat -c %u /tmp/c) -eq 0 ]"])?;
    sbx.shell(&["[ $(stat -c %g /tmp/c) -eq 1000 ]"])?;

    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_write_chown_both() -> RaptorResult<()> {
    colog::init();
    let mut sbx = spawn_sandbox("write_chown_both")?;

    write_etc_passwd(&mut sbx)?;
    write_etc_group(&mut sbx)?;

    sbx.write_file("/tmp/a", Some(Chown::new("root", "root")), None, b"Hello\n")?;
    sbx.shell(&["[ $(stat -c %u /tmp/a) -eq 0 ]"])?;
    sbx.shell(&["[ $(stat -c %g /tmp/a) -eq 0 ]"])?;

    sbx.write_file("/tmp/b", Some(Chown::new("user", "user")), None, b"Hello\n")?;
    sbx.shell(&["[ $(stat -c %u /tmp/b) -eq 1000 ]"])?;
    sbx.shell(&["[ $(stat -c %g /tmp/b) -eq 1000 ]"])?;

    sbx.write_file("/tmp/c", Some(Chown::new("root", "user")), None, b"Hello\n")?;
    sbx.shell(&["[ $(stat -c %u /tmp/c) -eq 0 ]"])?;
    sbx.shell(&["[ $(stat -c %g /tmp/c) -eq 1000 ]"])?;

    sbx.write_file("/tmp/d", Some(Chown::new("user", "root")), None, b"Hello\n")?;
    sbx.shell(&["[ $(stat -c %u /tmp/d) -eq 1000 ]"])?;
    sbx.shell(&["[ $(stat -c %g /tmp/d) -eq 0 ]"])?;

    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_write_chmod() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("write_chmod")?;

    let mut test_chmod = |mode: u32| -> RaptorResult<()> {
        const DATA: &[u8] = b"Hello world\n";
        sbx.write_file("/a", None, Some(mode), DATA)?;
        sbx.shell(&[&format!(
            "[ $(stat -c '%04a' /a) = {mode:04o} ] || {{ stat /a; stat -c '%04a' /a; exit 1; }}"
        )])?;
        sbx.shell(&[&format!(
            "[ $(stat -c '%s' /a) -eq {} ] || {{ stat /a; exit 1; }}; rm -f /a",
            DATA.len()
        )])?;
        Ok(())
    };

    test_chmod(0o0000)?; // ---------
    test_chmod(0o0777)?; // rwxrwxrwx
    test_chmod(0o0700)?; // rwx------
    test_chmod(0o0070)?; // ---rwx---
    test_chmod(0o0007)?; // ------rwx
    test_chmod(0o0750)?; // rwxr-x---
    test_chmod(0o0755)?; // rwxr-xr-x
    test_chmod(0o0775)?; // rwxrwxr-x

    test_chmod(0o1777)?; // rwxrwxrwt
    test_chmod(0o2777)?; // rwxrwsrwt
    test_chmod(0o4777)?; // rwsrwxrwx

    test_chmod(0o1000)?; // --------T
    test_chmod(0o2000)?; // -----S---
    test_chmod(0o4000)?; // --S------

    sbx.close()?;
    Ok(())
}
