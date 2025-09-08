use clap::Parser;

use dregistry::reference;

#[derive(Parser)]
struct Cli {
    #[arg(required = true)]
    names: Vec<String>,
}

fn main() {
    let args = Cli::parse();

    for name in args.names {
        eprintln!("Parsing [{name}]:");
        let source = reference::parse(&name).unwrap();
        eprintln!("{source:#?}");

        let output = source.to_string();
        assert_eq!(name, output);
    }
}
