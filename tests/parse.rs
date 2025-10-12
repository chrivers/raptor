use std::collections::BTreeMap;
use std::os::unix::fs::MetadataExt;
use std::sync::Arc;

use camino::{Utf8Path, Utf8PathBuf};
use minijinja::{Value, context};
use pretty_assertions::assert_eq;

use raptor::RaptorResult;
use raptor::dsl::Item;
use raptor::{dsl::Program, program::Loader};
use raptor_parser::ast::{
    Chown, FromSource, IncludeArg, InstEnvAssign, InstFrom, InstMkdir, InstMount, Instruction,
    MountOptions, MountType, Origin,
};

fn base_path() -> Utf8PathBuf {
    Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/cases/inst")
}

fn load_file(path: impl AsRef<Utf8Path>) -> RaptorResult<Arc<Program>> {
    let loader = Loader::new()?.with_base(base_path());

    loader.load_template(path, context! {})
}

fn assert_single_inst_eq(path: &Utf8Path, size: usize, res: &Program, inst: Instruction) {
    let origin = Origin::make(path, 0..size - 1);

    assert_eq!([Item::statement(inst, origin)], &res.code[..]);
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
fn parse_copy01() -> RaptorResult<()> {
    test_single_inst_parse("copy01.rapt", Instruction::copy(&["file"], "/foo"))
}

#[test]
fn parse_copy02() -> RaptorResult<()> {
    test_single_inst_parse(
        "copy02.rapt",
        Instruction::copy(&["a", "b", "c", "d"], "/dir/"),
    )
}

#[test]
fn parse_from01() -> RaptorResult<()> {
    test_single_inst_parse(
        "from01.rapt",
        Instruction::From(InstFrom {
            from: FromSource::Raptor("baselayer".into()),
        }),
    )
}

#[test]
fn parse_from02() -> RaptorResult<()> {
    test_single_inst_parse(
        "from02.rapt",
        Instruction::From(InstFrom {
            from: FromSource::Docker("debian:stable".into()),
        }),
    )
}

#[test]
fn parse_run01() -> RaptorResult<()> {
    test_single_inst_parse("run01.rapt", Instruction::run(&["id"]))
}

#[test]
fn parse_run02() -> RaptorResult<()> {
    test_single_inst_parse("run02.rapt", Instruction::run(&["ls", "-l"]))
}

#[test]
fn parse_run03() -> RaptorResult<()> {
    test_single_inst_parse(
        "run03.rapt",
        Instruction::run(&["/bin/sh", "-c", "echo 'foo'"]),
    )
}

#[test]
fn parse_write01() -> RaptorResult<()> {
    test_single_inst_parse("write01.rinc", Instruction::write("/foo", "bar"))
}

#[test]
fn parse_write02() -> RaptorResult<()> {
    test_single_inst_parse(
        "write02.rapt",
        Instruction::write("/foo", "bar").chown(Some(Chown::new("user", "group"))),
    )
}

#[test]
fn parse_mkdir01() -> RaptorResult<()> {
    test_single_inst_parse("mkdir01.rapt", Instruction::mkdir("/foo"))
}

#[test]
fn parse_mkdir02() -> RaptorResult<()> {
    test_single_inst_parse(
        "mkdir02.rapt",
        Instruction::Mkdir(InstMkdir {
            dest: "/foo/bar".into(),
            chmod: None,
            chown: None,
            parents: true,
        }),
    )
}

#[test]
fn parse_mkdir03() -> RaptorResult<()> {
    test_single_inst_parse(
        "mkdir03.rapt",
        Instruction::Mkdir(InstMkdir {
            dest: "/foo/bar".into(),
            chmod: Some(0o0755),
            chown: Some(Chown {
                user: Some("user".into()),
                group: Some("group".into()),
            }),
            parents: true,
        }),
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
        &[
            Item::statement(
                Instruction::include("render03", [IncludeArg::value("what", "world")]),
                Origin::make("render03.rapt", 0..29),
            ),
            Item::program(
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
                    Origin::make("render03.rinc", 0..43)
                )],
                context! { what => "world" },
                "render03.rinc",
            )
        ],
        program.code.as_slice()
    );

    Ok(())
}

#[test]
fn parse_entrypoint01() -> RaptorResult<()> {
    test_single_inst_parse("entrypoint01.rapt", Instruction::entrypoint(["/bin/sh"]))
}

#[test]
fn parse_entrypoint02() -> RaptorResult<()> {
    test_single_inst_parse(
        "entrypoint02.rapt",
        Instruction::entrypoint(["/bin/sh", "-c", "echo foo"]),
    )
}

#[test]
fn parse_cmd01() -> RaptorResult<()> {
    test_single_inst_parse("cmd01.rapt", Instruction::cmd(["/bin/sh"]))
}

#[test]
fn parse_cmd02() -> RaptorResult<()> {
    test_single_inst_parse(
        "cmd02.rapt",
        Instruction::cmd(["/bin/sh", "-c", "echo foo"]),
    )
}

#[test]
fn parse_include01() -> RaptorResult<()> {
    let program = load_file("include01.rapt")?;

    assert_eq!(
        &[
            Item::statement(
                Instruction::include("write01", []),
                Origin::make("include01.rapt", 0..15),
            ),
            Item::program(
                [Item::statement(
                    Instruction::write("/foo", "bar"),
                    Origin::make("write01.rinc", 0..16)
                )],
                context! {},
                "write01.rinc",
            )
        ],
        program.code.as_slice()
    );

    Ok(())
}

#[test]
fn parse_include02() -> RaptorResult<()> {
    let program = load_file("include02.rapt")?;

    assert_eq!(
        &[
            Item::statement(
                Instruction::include("include01", []),
                Origin::make("include02.rapt", 0..17),
            ),
            Item::program(
                [
                    Item::statement(
                        Instruction::include("write01", []),
                        Origin::make("include01.rinc", 0..15),
                    ),
                    Item::program(
                        [Item::statement(
                            Instruction::write("/foo", "bar"),
                            Origin::make("write01.rinc", 0..16)
                        )],
                        context! {},
                        "write01.rinc",
                    )
                ],
                context! {},
                "include01.rinc",
            )
        ],
        program.code.as_slice()
    );

    Ok(())
}

#[test]
fn parse_include03() -> RaptorResult<()> {
    let program = load_file("include03.rapt")?;

    assert_eq!(
        &[
            Item::statement(
                Instruction::include("include.run01", []),
                Origin::make("include03.rapt", 0..21),
            ),
            Item::program(
                [Item::statement(
                    Instruction::run(&["id"]),
                    Origin::make("include/run01.rinc", 0..6)
                )],
                context! {},
                "include/run01.rinc",
            )
        ],
        program.code.as_slice(),
    );

    Ok(())
}

#[test]
fn parse_include04() -> RaptorResult<()> {
    let program = load_file("include04.rapt")?;

    assert_eq!(
        &[
            Item::statement(
                Instruction::include("include.include01", []),
                Origin::make("include04.rapt", 0..25),
            ),
            Item::program(
                [
                    Item::statement(
                        Instruction::include("run01", []),
                        Origin::make("include/include01.rinc", 0..13),
                    ),
                    Item::program(
                        [Item::statement(
                            Instruction::run(&["id"]),
                            Origin::make("include/run01.rinc", 0..6)
                        )],
                        context! {},
                        "include/run01.rinc",
                    )
                ],
                context! {},
                "include/include01.rinc",
            )
        ],
        program.code.as_slice(),
    );

    Ok(())
}

#[test]
fn parse_expr01() -> RaptorResult<()> {
    test_single_inst_parse(
        "expr01.rapt",
        Instruction::render("foo", "bar", [IncludeArg::value("a", [1, 2, 3])]),
    )
}

#[test]
fn parse_expr02() -> RaptorResult<()> {
    test_single_inst_parse(
        "expr02.rapt",
        Instruction::render(
            "foo",
            "bar",
            [
                IncludeArg::value("a", [1, 2, 3]),
                IncludeArg::value("b", [4, 5, 6]),
            ],
        ),
    )
}

#[test]
fn parse_expr03() -> RaptorResult<()> {
    test_single_inst_parse(
        "expr03.rapt",
        Instruction::render(
            "foo",
            "bar",
            [IncludeArg::value("a", BTreeMap::<Value, Value>::new())],
        ),
    )
}

#[test]
fn parse_expr04() -> RaptorResult<()> {
    test_single_inst_parse(
        "expr04.rapt",
        Instruction::render(
            "foo",
            "bar",
            [IncludeArg::value("a", BTreeMap::from([("foo", "bar")]))],
        ),
    )
}

#[test]
fn parse_expr05() -> RaptorResult<()> {
    let mut map = BTreeMap::<Value, Value>::new();
    map.insert("foo".into(), Value::from_serialize([1, 2, 3]));
    map.insert("sub".into(), BTreeMap::from([("foo", "bar")]).into());

    test_single_inst_parse(
        "expr05.rapt",
        Instruction::render("foo", "bar", [IncludeArg::value("a", map)]),
    )
}

#[test]
fn parse_mount01() -> RaptorResult<()> {
    test_single_inst_parse(
        "mount01.rapt",
        Instruction::Mount(InstMount {
            opts: MountOptions {
                mtype: MountType::Simple,
            },
            name: "foo".into(),
            dest: "/bar".into(),
        }),
    )
}
