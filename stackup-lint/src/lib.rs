use graphql_parser;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::error::Error;

pub mod interface;
mod rules;
use interface::{CheckResult, Comment, Pos, PositionedComment, Severity};
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
    static ref REGEX: Regex = Regex::new(
        r"^schema parse error: Parse error at (?P<line>\d+):(?P<column>\d+)(?x)
        \n(?P<unexpected>.*)
        \n(?P<expected>.*)$",
    )
    .unwrap();
}

pub fn check(schema: &str) -> CheckResult {
    let document_result = graphql_parser::parse_schema(schema).map_err(|e| e.to_string());

    match document_result {
        Ok(document) => {
            let defns = document.definitions;

            let mut comments = Vec::new();

            comments.append(&mut check_associations(&defns));
            comments.append(&mut check_types_for_id_field(&defns));
            comments.append(&mut check_for_list_of_scalars(&defns));

            CheckResult::new(schema.to_string(), comments)
        }
        Err(e) => {
            let captures = REGEX.captures(e.trim()).expect("no captures");
            let line: usize = captures["line"].parse().unwrap();
            let column: usize = captures["column"].parse().unwrap();
            let expected = &captures["expected"];
            let unexpected = &captures["unexpected"];

            let message = format!(
                r"    {}
                    {}",
                unexpected, expected
            );

            let comment = Comment::new(Severity::Error, message);
            let p_comment = PositionedComment::new(Pos { line, column }, comment);
            let p_comments = vec![p_comment];
            CheckResult::new(schema.to_string(), p_comments)
        }
    }
}
