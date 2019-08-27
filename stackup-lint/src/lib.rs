use graphql_parser;
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::error::Error;

mod interface;
mod rules;
use interface::CheckResult;
use rules::{
    associations::check_associations, id::check_types_for_id_field,
    list_of_scalars::check_for_list_of_scalars,
};

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

lazy_static! {
    pub(crate) static ref SCALARS: HashSet<String> = {
        let mut ss = HashSet::new();
        ss.insert("ID".to_string());
        ss.insert("Boolean".to_string());
        ss.insert("String".to_string());
        ss.insert("Int".to_string());
        ss.insert("Float".to_string());
        ss.insert("Decimal".to_string());
        ss.insert("Date".to_string());
        ss.insert("DateTime".to_string());
        ss.insert("File".to_string());
        ss
    };
}

pub fn check(schema: &str) -> Result<CheckResult> {
    let document = graphql_parser::parse_schema(schema).map_err(|e| e.to_string())?;

    let defns = document.definitions;

    let mut comments = Vec::new();

    comments.append(&mut check_associations(&defns));
    comments.append(&mut check_types_for_id_field(&defns));
    comments.append(&mut check_for_list_of_scalars(&defns));

    Ok(CheckResult::new(schema.to_string(), comments))
}
