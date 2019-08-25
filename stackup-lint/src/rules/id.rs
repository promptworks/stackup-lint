use crate::interface::{Comment, PositionedComment, Severity};
use graphql_parser::{
    self,
    query::Type,
    schema::{Definition, Field, ObjectType, TypeDefinition},
    Pos,
};

pub(crate) fn check_types_for_id_field(defns: &[Definition]) -> Vec<PositionedComment> {
    defns
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
        .flat_map(|(fields, object_name, position)| {
            let id_fields: Vec<_> = fields.iter().filter(|f| f.name == "id").collect();
            check_id_fields(*position, &object_name, &id_fields)
        })
        .collect()
}

fn check_id_fields(
    position: Pos,
    object_name: &str,
    id_fields: &[&Field],
) -> Option<PositionedComment> {
    match id_fields.len() {
        0 => {
            let message = format!(
                "Missing id field on object type {}, consider adding one",
                object_name
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
                position, object_name
            );
            let comment = Comment::new(Severity::Error, message);
            Some(PositionedComment::new(position, comment))
        }
    }
}
