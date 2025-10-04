use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use camino::Utf8Path;
use libtest_mimic::{Arguments, Trial};
use pretty_assertions::assert_eq;
use raptor::RaptorResult;

// The function `cargo_dir` is copied from cargo source code (dual Apache / MIT license)
pub fn cargo_dir() -> PathBuf {
    env::var_os("CARGO_BIN_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            env::current_exe().ok().map(|mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            })
        })
        .unwrap_or_else(|| panic!("CARGO_BIN_PATH wasn't set. Cannot continue running test"))
}

fn run_raptor(filename: &Utf8Path) -> RaptorResult<String> {
    let mut proc = Command::new(cargo_dir().join("raptor"))
        .arg("build")
        .arg("-n")
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
    let args = Arguments::from_args();

    let mut tests = vec![];

    let cases = Utf8Path::new("tests/cases/error").read_dir_utf8()?;

    for dent in cases {
        let dent = dent?;
        let file_name = dent.file_name();
        if !file_name.ends_with(".rapt") || !file_name.starts_with("error_") {
            continue;
        }
        let refpath = dent.path().with_extension("out");

        let path = dent.path().to_owned();
        let test = Trial::test(dent.file_name(), move || {
            let refdata = std::fs::read_to_string(refpath)?;
            let newdata = run_raptor(&path)?;

            assert_eq!(refdata, newdata);
            Ok(())
        });

        tests.push(test);
    }

    /* run all tests using libtest-mimic */
    libtest_mimic::run(&args, tests).exit()
}
