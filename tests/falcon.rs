use std::os::unix::net::UnixListener;
use std::process::Command;

use camino_tempfile::{NamedUtf8TempFile, Utf8TempDir};
use nix::errno::Errno;

use raptor::sandbox::{FalconClient, SandboxExt};
use raptor::{RaptorError, RaptorResult};

const TEST_DATA: &[u8] = b"Raptortest\n";

fn spawn_client() -> RaptorResult<FalconClient> {
    let exe = std::env::current_exe().unwrap();
    let deps = exe.parent().unwrap().parent().unwrap();
    let client = deps.join("falcon");

    let tempdir = Utf8TempDir::new()?;
    let socket_path = tempdir.path().join("raptor-test");
    let listen = UnixListener::bind(&socket_path)?;

    let proc = Command::new(client)
        .env("FALCON_LOG_LEVEL", "off")
        .env("FALCON_SOCKET", socket_path.as_str())
        .spawn()?;

    let conn = listen.accept()?.0;

    Ok(FalconClient::new(proc, conn))
}

#[test]
fn chdir_tmp() -> RaptorResult<()> {
    let mut sc = spawn_client()?;
    sc.chdir("/tmp")?;
    sc.close()
}

#[test]
fn write_file_over_dir() -> RaptorResult<()> {
    let mut sc = spawn_client()?;
    match sc.create_file("/tmp".into(), None, None).unwrap_err() {
        RaptorError::SandboxRequestError(Errno::EISDIR) => {}
        err => panic!("unexpected error: {err}"),
    }
    sc.close()
}

#[test]
fn client_write_file() -> RaptorResult<()> {
    let mut sc = spawn_client()?;

    let tmpfile = NamedUtf8TempFile::new()?;
    let path = tmpfile.path();

    sc.write_file(path, None, None, TEST_DATA)?;

    assert_eq!(std::fs::read(path)?, TEST_DATA);

    sc.close()
}
