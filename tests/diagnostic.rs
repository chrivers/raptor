use std::{
    io::Read,
    process::{Command, Stdio},
};

use camino::Utf8Path;
use libtest_mimic::{Arguments, Trial};
use raptor::RaptorResult;

fn run_raptor(filename: &Utf8Path) -> RaptorResult<String> {
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
    let args = Arguments::from_args();

    let mut tests = vec![];

    let cases = Utf8Path::new("tests/cases/error").read_dir_utf8()?;

    for dent in cases {
        let dent = dent?;
        let file_name = dent.file_name();
        if !file_name.ends_with(".rapt") || !file_name.starts_with("error-") {
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
