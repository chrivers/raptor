pub mod ast;

#[derive(pest_consume::Parser)]
#[grammar = "raptorfile.pest"]
pub struct RaptorFileParser;
