use std::os::unix::fs::MetadataExt;

use camino::{Utf8Path, Utf8PathBuf};
use minijinja::context;

use raptor::dsl::{Chown, IncludeArg, InstEnvAssign, Instruction, Item, Origin, Program};
use raptor::program::Loader;
use raptor::RaptorResult;

fn base_path() -> Utf8PathBuf {
    Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/inst")
}

fn load_file(path: impl AsRef<Utf8Path>) -> RaptorResult<Program> {
    let mut loader = Loader::new(base_path(), false)?;

    loader.parse_template(path, &context! {})
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
    test_single_inst_parse("write01.rapt", Instruction::write("/foo", "bar"))
}

#[test]
fn parse_write02() -> RaptorResult<()> {
    test_single_inst_parse(
        "write02.rapt",
        Instruction::write("/foo", "bar").chown(Some(Chown::new("user", "group"))),
    )
}

#[test]
fn parse_env01() -> RaptorResult<()> {
    test_single_inst_parse(
        "env01.rapt",
        Instruction::env([InstEnvAssign::new("foo", "bar")]),
    )
}

#[test]
fn parse_env02() -> RaptorResult<()> {
    test_single_inst_parse(
        "env02.rapt",
        Instruction::env([
            InstEnvAssign::new("foo1", "bar1"),
            InstEnvAssign::new("foo2", "bar2"),
        ]),
    )
}

#[test]
fn parse_workdir01() -> RaptorResult<()> {
    test_single_inst_parse("workdir01.rapt", Instruction::workdir("/foo"))
}

#[test]
fn parse_render01() -> RaptorResult<()> {
    test_single_inst_parse(
        "render01.rapt",
        Instruction::render("include/template01.tmpl", "/a", []),
    )
}

#[test]
fn parse_render02() -> RaptorResult<()> {
    test_single_inst_parse(
        "render02.rapt",
        Instruction::render(
            "include/template02.tmpl",
            "/a",
            [IncludeArg::value("what", "world")],
        ),
    )
}

#[test]
fn parse_render03() -> RaptorResult<()> {
    let program = load_file("render03.rapt")?;

    assert_eq!(
        &program.code,
        &[Item::program(
            [Item::statement(
                Instruction::render(
                    "include/template02.tmpl",
                    "/a",
                    [IncludeArg::lookup(
                        "what",
                        &["what"],
                        Origin::make("render03.rinc", 39..43),
                    )],
                ),
                Origin::make("render03.rinc", 0..44)
            )],
            context! { what => "world" },
            "render03.rinc",
        )]
    );

    Ok(())
}

#[test]
fn parse_include01() -> RaptorResult<()> {
    let program = load_file("include01.rapt")?;

    assert_eq!(
        &program.code,
        &[Item::program(
            [Item::statement(
                Instruction::write("/foo", "bar"),
                Origin::make("write01.rapt", 0..17)
            )],
            context! {},
            "write01.rapt",
        )]
    );

    Ok(())
}

#[test]
fn parse_include02() -> RaptorResult<()> {
    let program = load_file("include02.rapt")?;

    assert_eq!(
        &program.code,
        &[Item::program(
            [Item::program(
                [Item::statement(
                    Instruction::write("/foo", "bar"),
                    Origin::make("write01.rapt", 0..17)
                )],
                context! {},
                "write01.rapt",
            )],
            context! {},
            "include01.rapt",
        )]
    );

    Ok(())
}

#[test]
fn parse_include03() -> RaptorResult<()> {
    let program = load_file("include03.rapt")?;

    assert_eq!(
        &program.code,
        &[Item::program(
            [Item::statement(
                Instruction::run(&["id"]),
                Origin::make("include/run01.rinc", 0..7)
            )],
            context! {},
            "include/run01.rinc",
        )]
    );

    Ok(())
}

#[test]
fn parse_include04() -> RaptorResult<()> {
    let program = load_file("include04.rapt")?;

    assert_eq!(
        &program.code,
        &[Item::program(
            [Item::program(
                [Item::statement(
                    Instruction::run(&["id"]),
                    Origin::make("include/run01.rinc", 0..7)
                )],
                context! {},
                "include/run01.rinc",
            )],
            context! {},
            "include/include01.rinc",
        )]
    );

    Ok(())
}
