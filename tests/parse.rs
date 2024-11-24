use std::os::unix::fs::MetadataExt;
use std::sync::Arc;

use camino::Utf8Path;
use minijinja::context;

use raptor::dsl::{
    Chown, InstEnv, InstEnvAssign, InstWorkdir, InstWrite, Instruction, Origin, Statement,
};
use raptor::program::{Loader, Program};
use raptor::template::make_environment;
use raptor::RaptorResult;

fn load_file(name: &str) -> RaptorResult<Program> {
    let mut loader = Loader::new(make_environment()?, false);
    loader.parse_template(name, &context! {})
}

fn assert_single_inst_eq(path: &str, size: usize, res: &Program, inst: Instruction) {
    let origin = Origin {
        path: Arc::new(path.into()),
        span: 0..size,
    };

    assert_eq!(&res.0, &[Statement { inst, origin }]);
}

#[allow(clippy::cast_possible_truncation)]
fn test_single_inst_parse(filename: &str, inst: Instruction) -> RaptorResult<()> {
    let path = Utf8Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cases")
        .join(filename);
    let size = std::fs::File::open(&path)?.metadata()?.size();
    let res = load_file(path.as_str())?;
    assert_single_inst_eq(path.as_str(), size as usize, &res, inst);
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
