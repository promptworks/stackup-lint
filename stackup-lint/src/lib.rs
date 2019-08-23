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

    let defns = document.definitions;

    let mut comments = Vec::new();

    comments.append(&mut check_associations(&defns));
    comments.extend(defns.into_iter().filter_map(has_id).flatten());

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

fn check_associations(defns: &[Definition]) -> Vec<PositionedComment> {
    let object_defns: Vec<_> = defns
        .iter()
        .filter_map(|defn| match defn {
            Definition::TypeDefinition(TypeDefinition::Object(ObjectType {
                fields,
                name,
                position,
                ..
            })) => Some((fields, name, position)),
            _ => None,
        })
        .collect();
    let names: HashSet<_> = object_defns.iter().map(|(_, name, _)| *name).collect();
    object_defns
        .into_iter()
        .flat_map(|(fields, name, position)| fields.iter().map(move |f| (f, name, position)))
        .filter(|(f, _, _)| match f.field_type {
            Type::NamedType(ref type_name) if names.contains(type_name) => true,
            Type::NonNullType(ref inner_type) => {
                if let Type::NamedType(ref type_name) = **inner_type {
                    names.contains(type_name)
                } else {
                    false
                }
            }
            _ => false,
        })
        .filter(|(f, _, _)| !f.directives.iter().any(|d| &d.name == "belongsTo"))
        .map(|(f, _, _)| {
            let message = r#"Missing "@belongsTo" directive"#;
            let comment = Comment::new(Severity::Error, message.to_string());
            PositionedComment::new(f.position, comment)
        })
        .collect()
}
