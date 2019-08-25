use crate::interface::{Comment, PositionedComment, Severity};
use graphql_parser::{
    self,
    query::Type,
    schema::{Definition, Field, ObjectType, TypeDefinition},
    Pos,
};

pub(crate) fn has_id(defn: Definition) -> Option<Vec<PositionedComment>> {
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
