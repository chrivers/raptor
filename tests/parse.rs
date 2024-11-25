use std::os::unix::fs::MetadataExt;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use minijinja::{context, Value};

use raptor::dsl::{
    Chown, IncludeArg, IncludeArgValue, InstEnv, InstEnvAssign, InstRender, InstWorkdir, InstWrite,
    Instruction, Item, Lookup, Origin, Statement,
};
use raptor::program::{Loader, Program};
use raptor::template::make_environment;
use raptor::RaptorResult;

fn test_path(filename: &str) -> Utf8PathBuf {
    Utf8Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cases")
        .join(filename)
}

fn load_file(path: &Utf8Path) -> RaptorResult<Program> {
    let mut loader = Loader::new(make_environment()?, false);
    loader.parse_template(path.as_str(), &context! {})
}

fn assert_single_inst_eq(path: &Utf8Path, size: usize, res: &Program, inst: Instruction) {
    let origin = Origin {
        path: Arc::new(path.to_string()),
        span: 0..size,
    };

    assert_eq!(&res.code, &[Item::Statement(Statement { inst, origin })]);
}

#[allow(clippy::cast_possible_truncation)]
fn test_single_inst_parse(filename: &str, inst: Instruction) -> RaptorResult<()> {
    let path = test_path(filename);
    let program = load_file(&path)?;
    let size = std::fs::File::open(&path)?.metadata()?.size();
    assert_single_inst_eq(&path, size as usize, &program, inst);
    Ok(())
}

#[test]
fn test_parse_write01() -> RaptorResult<()> {
    test_single_inst_parse(
        "write01.rapt",
        Instruction::Write(InstWrite {
            dest: "/foo".into(),
            body: "bar".into(),
            chmod: None,
            chown: None,
        }),
    )
}

#[test]
fn test_parse_write02() -> RaptorResult<()> {
    test_single_inst_parse(
        "write02.rapt",
        Instruction::Write(InstWrite {
            dest: "/foo".into(),
            body: "bar".into(),
            chmod: None,
            chown: Some(Chown {
                user: Some("user".into()),
                group: Some("group".into()),
            }),
        }),
    )
}

#[test]
fn test_parse_env01() -> RaptorResult<()> {
    test_single_inst_parse(
        "env01.rapt",
        Instruction::Env(InstEnv {
            env: vec![InstEnvAssign {
                key: "foo".into(),
                value: "bar".into(),
            }],
        }),
    )
}

#[test]
fn test_parse_env02() -> RaptorResult<()> {
    test_single_inst_parse(
        "env02.rapt",
        Instruction::Env(InstEnv {
            env: vec![
                InstEnvAssign {
                    key: "foo1".into(),
                    value: "bar1".into(),
                },
                InstEnvAssign {
                    key: "foo2".into(),
                    value: "bar2".into(),
                },
            ],
        }),
    )
}

#[test]
fn test_parse_workdir01() -> RaptorResult<()> {
    test_single_inst_parse(
        "workdir01.rapt",
        Instruction::Workdir(InstWorkdir { dir: "/foo".into() }),
    )
}
