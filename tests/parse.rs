use std::os::unix::fs::MetadataExt;

use camino::{Utf8Path, Utf8PathBuf};
use minijinja::{context, Value};

use raptor::dsl::{
    Chown, IncludeArg, IncludeArgValue, InstEnv, InstEnvAssign, InstRender, InstRun, InstWorkdir,
    InstWrite, Instruction, Item, Lookup, Origin, Program,
};
use raptor::program::Loader;
use raptor::RaptorResult;

fn base_path() -> Utf8PathBuf {
    Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/inst")
}

fn load_file(path: impl AsRef<Utf8Path>) -> RaptorResult<Program> {
    let mut loader = Loader::new(base_path(), false)?;
    loader.parse_template(dbg!(path.as_ref().file_name().unwrap()), &context! {})
}

fn assert_single_inst_eq(path: &Utf8Path, size: usize, res: &Program, inst: Instruction) {
    let origin = Origin::make(path, 0..size);

    assert_eq!(&res.code, &[Item::statement(inst, origin)]);
}

#[allow(clippy::cast_possible_truncation)]
fn test_single_inst_parse(filename: &str, inst: Instruction) -> RaptorResult<()> {
    let program = load_file(filename)?;
    let size = std::fs::File::open(base_path().join(filename))?
        .metadata()?
        .size();
    assert_single_inst_eq(filename.into(), size as usize, &program, inst);
    Ok(())
}

#[test]
fn parse_write01() -> RaptorResult<()> {
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
fn parse_write02() -> RaptorResult<()> {
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
fn parse_env01() -> RaptorResult<()> {
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
fn parse_env02() -> RaptorResult<()> {
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
fn parse_workdir01() -> RaptorResult<()> {
    test_single_inst_parse(
        "workdir01.rapt",
        Instruction::Workdir(InstWorkdir { dir: "/foo".into() }),
    )
}

#[test]
fn parse_render01() -> RaptorResult<()> {
    test_single_inst_parse(
        "render01.rapt",
        Instruction::Render(InstRender {
            src: "include/template01.tmpl".into(),
            dest: "/a".into(),
            chmod: None,
            chown: None,
            args: vec![],
        }),
    )
}

#[test]
fn parse_render02() -> RaptorResult<()> {
    test_single_inst_parse(
        "render02.rapt",
        Instruction::Render(InstRender {
            src: "include/template02.tmpl".into(),
            dest: "/a".into(),
            chmod: None,
            chown: None,
            args: vec![IncludeArg {
                name: "what".into(),
                value: IncludeArgValue::Value(Value::from("world")),
            }],
        }),
    )
}

#[test]
fn parse_render03() -> RaptorResult<()> {
    let program = load_file("render03.rapt")?;

    let name = "what".into();

    let value = IncludeArgValue::Lookup(Lookup::new(
        vec!["what".into()],
        Origin::make("render03.rinc", 39..43),
    ));

    let inst = Instruction::Render(InstRender {
        src: "include/template02.tmpl".into(),
        dest: "/a".into(),
        chmod: None,
        chown: None,
        args: vec![IncludeArg { name, value }],
    });

    let origin = Origin::make("render03.rinc", 0..44);

    let code = vec![Item::statement(inst, origin)];

    let ctx = context! { what => "world" };

    assert_eq!(&program.code, &[Item::program(code, ctx)]);

    Ok(())
}
