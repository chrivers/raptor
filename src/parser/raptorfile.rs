use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "raptorfile.pest"]
pub struct RaptorFileParser;
