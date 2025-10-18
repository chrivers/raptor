use std::ops::{Deref, DerefMut};

use camino::Utf8Path;
use camino_tempfile::Utf8TempDir;
use raptor::sandbox::{FalconClient, Sandbox, SandboxExt};
use raptor::util::link_or_copy_file;
use raptor::{RaptorError, RaptorResult};
use raptor_parser::ast::Chown;

const BUSYBOX_PATH: &str = "/bin/busybox";

/* wrapper type for Sandbox, which keeps tempdir alive as long as sandbox is */
/* used. Implements transparent deref to make use of it seamless. */
#[allow(dead_code)]
struct SandboxWrapper {
    sandbox: Sandbox,
    tempdir: Utf8TempDir,
}

impl Deref for SandboxWrapper {
    type Target = Sandbox;

    fn deref(&self) -> &Self::Target {
        &self.sandbox
    }
}

impl DerefMut for SandboxWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sandbox
    }
}

fn spawn_sandbox(name: &str) -> RaptorResult<SandboxWrapper> {
    assert!(
        std::fs::exists(BUSYBOX_PATH)?,
        "Busybox not required for testing, but not found at {BUSYBOX_PATH:?}"
    );

    /* construct temporary directory with busybox as /bin/sh */
    let tempdir = Utf8TempDir::new()?;
    std::fs::create_dir(tempdir.path().join("bin"))?;
    link_or_copy_file(BUSYBOX_PATH, tempdir.path().join("bin/sh"))?;

    let rootdir = Utf8Path::new("tests/output").join(name);

    let builder = Sandbox::builder().sudo(true);
    let sandbox = Sandbox::custom(
        builder,
        &[tempdir.path()],
        &rootdir,
        &Sandbox::find_falcon_dev().unwrap(),
    )?;

    Ok(SandboxWrapper { sandbox, tempdir })
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
    let client = sbx.client();

    client.shell(&["true"])?;

    assert!(matches!(
        client.shell(&["false"]).unwrap_err(),
        RaptorError::SandboxRunError(es) if es.code() == Some(1)
    ));
    assert!(matches!(
        client.shell(&["exit 100"]).unwrap_err(),
        RaptorError::SandboxRunError(es) if es.code() == Some(100)
    ));
    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_workdir() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("workdir")?;
    let client = sbx.client();
    client.shell(&["[ $PWD == / ]"])?;
    client.chdir("/bin")?;
    client.shell(&["[ $PWD == /bin ]"])?;
    client.chdir("../usr")?;
    client.shell(&["[ $PWD == /usr ]"])?;
    client.chdir("..")?;
    client.shell(&["[ $PWD == / ]"])?;
    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_setenv() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("setenv")?;
    let client = sbx.client();
    client.shell(&["[ x${FOO} == x ]"])?;
    client.setenv("FOO", "BAR")?;
    client.shell(&["[ x${FOO} == xBAR ]"])?;
    client.setenv("FOO", "OTHER")?;
    client.shell(&["[ x${FOO} == xOTHER ]"])?;
    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_write_data() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("write_data")?;
    let client = sbx.client();

    client.write_file("/tmp/a", None, None, b"Hello world\n")?;
    client.write_file(
        "/tmp/b",
        None,
        None,
        "f0ef7081e1539ac00ef5b761b4fb01b3  a\n",
    )?;

    client.shell(&["cd /tmp && md5sum -cs b"])?;

    sbx.close()?;
    Ok(())
}

fn write_etc_passwd(client: &mut FalconClient) -> RaptorResult<()> {
    client.write_file(
        "/etc/passwd",
        None,
        None,
        concat!(
            "root:x:0:0:root:/root:/bin/sh\n",
            "user:x:1000:1000:user:/home/user:/bin/sh\n"
        ),
    )
}

fn write_etc_group(client: &mut FalconClient) -> RaptorResult<()> {
    client.write_file(
        "/etc/group",
        None,
        None,
        concat!("root:x:0:\n", "user:x:1000:\n"),
    )
}

#[test]
fn nspawn_write_chown_user() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("write_chown_user")?;
    let client = sbx.client();

    write_etc_passwd(client)?;

    client.write_file("/tmp/c", Some(Chown::user("root")), None, b"Hello world\n")?;
    client.shell(&["[ $(stat -c %u /tmp/c) -eq 0 ]"])?;
    client.shell(&["[ $(stat -c %g /tmp/c) -eq 0 ]"])?;

    client.write_file("/tmp/c", Some(Chown::user("user")), None, b"Hello world\n")?;
    client.shell(&["[ $(stat -c %u /tmp/c) -eq 1000 ]"])?;
    client.shell(&["[ $(stat -c %g /tmp/c) -eq 0 ]"])?;

    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_write_chown_group() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("write_chown_group")?;
    let client = sbx.client();

    write_etc_passwd(client)?;
    write_etc_group(client)?;

    client.write_file("/tmp/c", Some(Chown::group("root")), None, b"Hello world\n")?;
    client.shell(&["[ $(stat -c %u /tmp/c) -eq 0 ]"])?;
    client.shell(&["[ $(stat -c %g /tmp/c) -eq 0 ]"])?;

    client.write_file("/tmp/c", Some(Chown::group("user")), None, b"Hello world\n")?;
    client.shell(&["[ $(stat -c %u /tmp/c) -eq 0 ]"])?;
    client.shell(&["[ $(stat -c %g /tmp/c) -eq 1000 ]"])?;

    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_write_chown_both() -> RaptorResult<()> {
    colog::init();
    let mut sbx = spawn_sandbox("write_chown_both")?;
    let client = sbx.client();

    write_etc_passwd(client)?;
    write_etc_group(client)?;

    client.write_file("/tmp/a", Some(Chown::new("root", "root")), None, b"Hello\n")?;
    client.shell(&["[ $(stat -c %u /tmp/a) -eq 0 ]"])?;
    client.shell(&["[ $(stat -c %g /tmp/a) -eq 0 ]"])?;

    client.write_file("/tmp/b", Some(Chown::new("user", "user")), None, b"Hello\n")?;
    client.shell(&["[ $(stat -c %u /tmp/b) -eq 1000 ]"])?;
    client.shell(&["[ $(stat -c %g /tmp/b) -eq 1000 ]"])?;

    client.write_file("/tmp/c", Some(Chown::new("root", "user")), None, b"Hello\n")?;
    client.shell(&["[ $(stat -c %u /tmp/c) -eq 0 ]"])?;
    client.shell(&["[ $(stat -c %g /tmp/c) -eq 1000 ]"])?;

    client.write_file("/tmp/d", Some(Chown::new("user", "root")), None, b"Hello\n")?;
    client.shell(&["[ $(stat -c %u /tmp/d) -eq 1000 ]"])?;
    client.shell(&["[ $(stat -c %g /tmp/d) -eq 0 ]"])?;

    sbx.close()?;
    Ok(())
}

#[test]
fn nspawn_write_chmod() -> RaptorResult<()> {
    let mut sbx = spawn_sandbox("write_chmod")?;
    let client = sbx.client();

    let mut test_chmod = |mode: u32| -> RaptorResult<()> {
        const DATA: &[u8] = b"Hello world\n";
        client.write_file("/a", None, Some(mode), DATA)?;
        client.shell(&[&format!(
            "[ $(stat -c '%04a' /a) = {mode:04o} ] || {{ stat /a; stat -c '%04a' /a; exit 1; }}"
        )])?;
        client.shell(&[&format!(
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
