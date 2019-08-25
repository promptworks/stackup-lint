use crate::interface::{Comment, PositionedComment, Severity};
use graphql_parser::{
    self,
    query::Type,
    schema::{Definition, Field, ObjectType, TypeDefinition},
    Pos,
};
use std::collections::HashSet;

pub(crate) fn check_associations(defns: &[Definition]) -> Vec<PositionedComment> {
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
    let object_names: HashSet<_> = object_defns.iter().map(|(_, name, _)| *name).collect();

    let fields_with_associations = object_defns
        .into_iter()
        .flat_map(|(fields, object_name, position)| {
            fields.iter().map(move |f| (f, object_name, position))
        })
        .filter(|(f, _, _)| match f.field_type {
            Type::NamedType(ref type_name) if object_names.contains(type_name) => true,
            Type::NonNullType(ref inner_type) => {
                if let Type::NamedType(ref type_name) = **inner_type {
                    object_names.contains(type_name)
                } else {
                    false
                }
            }
            _ => false,
        });

    check_belongs_to(fields_with_associations.clone())
}

fn check_belongs_to<'a, I>(fields_with_associations: I) -> Vec<PositionedComment>
where
    I: IntoIterator<Item = (&'a Field, &'a String, &'a Pos)>,
{
    fields_with_associations
        .into_iter()
        .filter(|(f, _, _)| !f.directives.iter().any(|d| &d.name == "belongsTo"))
        .map(|(f, _, _)| {
            let message = r#"Missing "@belongsTo" directive"#;
            let comment = Comment::new(Severity::Error, message.to_string());
            PositionedComment::new(f.position, comment)
        })
        .collect()
}
