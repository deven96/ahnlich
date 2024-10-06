use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "syntax/syntax.pest"]
pub(crate) struct QueryParser;
