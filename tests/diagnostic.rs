use std::{
    io::Read,
    process::{Command, Stdio},
};

use log::{error, info, warn};

use camino::Utf8Path;
use raptor::RaptorResult;

fn run_raptor(filename: &str) -> RaptorResult<String> {
    let mut proc = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("--bin")
        .arg("raptor")
        .arg(filename)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdout = proc.stdout.take().unwrap();
    let mut message = String::new();
    stdout.read_to_string(&mut message)?;

    Ok(message)
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn main() -> RaptorResult<()> {
    colog::init();

    let cases = Utf8Path::new("tests/cases/error").read_dir_utf8()?;

    let mut passed = 0;

    let mut failed = 0;
    for dent in cases {
        let dent = dent?;
        if !dent.file_name().ends_with(".rapt") {
            continue;
        }

        let refpath = dent.path().with_extension("out");
        let refdata = std::fs::read_to_string(refpath);

        let newdata = run_raptor(dent.path().as_str());

        match (refdata, newdata) {
            (Ok(a), Ok(b)) => {
                if a == b {
                    info!("[{:<60}] OK", dent.path());
                    passed += 1;
                } else {
                    error!("[{}] Failed:", dent.path());
                    error!("--[ REFERENCE ]-----------------------------------------------------------------");
                    eprintln!("{a}");
                    error!("--[ GENERATED ]-----------------------------------------------------------------");
                    eprintln!("{b}");
                    error!("--------------------------------------------------------------------------------");
                    failed += 1;
                }
            }
            (Ok(_), Err(err)) => {
                error!("Failed to run raptor: {err}");
                failed += 1;
            }
            (Err(err), Ok(_)) => {
                error!("Failed to load reference data: {err}");
                failed += 1;
            }
            (Err(a), Err(b)) => {
                error!("Failed to run raptor: {a}");
                error!("Failed to load reference data: {b}");
                failed += 1;
            }
        }
    }

    if failed > 0 {
        warn!("{passed} passed, {failed} failed");
        std::process::exit(1);
    }

    eprintln!("------------------------------------------------------------------");
    eprintln!("Result: all {passed} tests pass");
    eprintln!();
    Ok(())
}
