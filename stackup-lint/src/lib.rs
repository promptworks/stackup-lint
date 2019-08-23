use graphql_parser::{self};
use std::error::Error;

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

pub fn check(schema: &str) -> Result<()> {
    let _document =
        graphql_parser::parse_schema(schema).map_err(|_| "could not parse schema".to_string())?;

    Ok(())
}
