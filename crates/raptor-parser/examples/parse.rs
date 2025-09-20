use std::sync::Arc;

use camino::Utf8PathBuf;
#[allow(clippy::wildcard_imports)]
use raptor_parser::parser::*;

#[macro_use]
extern crate log;

#[allow(clippy::needless_raw_string_hashes)]
fn main() {
    colog::init();

    let filename = Arc::new(Utf8PathBuf::from("<input>"));

    let tests = [
        r#"true"#,                             //
        r#"false"#,                            //
        r#"0"#,                                //
        r#"1"#,                                //
        r#""foo\tbar""#,                       //
        r#"1234"#,                             //
        r#""foo""#,                            //
        r#"[]"#,                               //
        r#"[1,2,3]"#,                          //
        r#"[true, false, 123]"#,               //
        r#"{}"#,                               //
        r#"{1: 123}"#,                         //
        r#"{"a": "b"}"#,                       //
        r#"{"a": ["b", true, [{"x": "y"}]]}"#, //
        r#"x"#,                                //
        r#"x.y"#,                              //
        r#"x.y.z"#,                            //
    ];

    for test in tests {
        let res = ExpressionParser::new().parse(&filename, test);
        match res {
            Ok(res) => println!("{res}"),
            Err(err) => {
                error!("Failed test: {test:?}");
                error!(">> {err}");
            }
        }
    }

    let res = FileParser::new().parse(&filename, "WORKDIR foo\n");
    println!("{res:?}");
}
