use graphql_parser::{
    self,
    query::Type,
    schema::{Definition, Field, ObjectType, TypeDefinition},
    Pos,
};
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::error::Error;

mod interface;
use interface::*;

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

lazy_static! {
    static ref SCALARS: HashSet<String> = {
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
    let document =
        graphql_parser::parse_schema(schema).map_err(|_| "could not parse schema".to_string())?;
    let comments: Vec<_> = document
        .definitions
        .into_iter()
        .filter_map(has_id)
        .flatten()
        .collect();

    Ok(CheckResult::new(schema.to_string(), comments))
}

fn has_id(defn: Definition) -> Option<Vec<PositionedComment>> {
    if let Definition::TypeDefinition(TypeDefinition::Object(ObjectType {
        fields,
        name,
        position,
        ..
    })) = defn
    {
        let (id_fields, other_fields): (Vec<_>, Vec<_>) =
            fields.iter().partition(|f| f.name == "id");
        let mut c = check_id_fields(position, &name, &id_fields);
        let mut comments = check_fields_for_id(&other_fields);

        if let Some(comment) = c.take() {
            comments.push(comment);
        }

        Some(comments)
    } else {
        None
    }
}

fn check_id_fields(position: Pos, name: &str, id_fields: &[&Field]) -> Option<PositionedComment> {
    match id_fields.len() {
        0 => {
            let message = format!(
                "Missing id field on object type {}, consider adding one",
                name
            );
            let comment = Comment::new(Severity::Error, message);
            Some(PositionedComment::new(position, comment))
        }
        1 => {
            let id_field = id_fields.first().unwrap();
            let make_comment = || {
                let message = r#"Consider making this "id: ID!""#;
                let comment = Comment::new(Severity::Warning, message.to_string());
                PositionedComment::new(position, comment)
            };
            match id_field.field_type {
                Type::NamedType(ref type_name) if type_name == "ID" => Some(make_comment()),
                Type::ListType(_) => Some(make_comment()),
                Type::NonNullType(ref inner_type) => {
                    if let Type::NamedType(ref id) = **inner_type {
                        if id != "ID" {
                            Some(make_comment())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => Some(make_comment()),
            }
        }
        _ => {
            let message = format!(
                "{} multiple fields with the same name on object type {}",
                position, name
            );
            let comment = Comment::new(Severity::Error, message);
            Some(PositionedComment::new(position, comment))
        }
    }
}

fn check_fields_for_id(fields: &[&Field]) -> Vec<PositionedComment> {
    fields
        .iter()
        .filter_map(|f| {
            let make_comment = || {
                let message = r#"Consider making this "id: ID!""#;
                let comment = Comment::new(Severity::Warning, message.to_string());
                PositionedComment::new(f.position, comment)
            };
            match f.field_type {
                Type::NamedType(ref type_name) if type_name == "ID" => Some(make_comment()),
                Type::NonNullType(ref inner_type) => {
                    if let Type::NamedType(ref id) = **inner_type {
                        if id == "ID" {
                            Some(make_comment())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        })
        .collect()
}
