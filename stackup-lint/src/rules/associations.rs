use super::ObjectDefn;
use crate::interface::{Comment, PositionedComment, Severity};
use graphql_parser::{
    self,
    query::Type,
    schema::{Definition, Field},
};
use heck::MixedCase;
use std::collections::HashMap;

struct FieldWithAssociation<'a> {
    field: &'a Field,
    object_defn: &'a ObjectDefn<'a>,
}

impl<'a> FieldWithAssociation<'a> {
    fn new(field: &'a Field, object_defn: &'a ObjectDefn) -> Self {
        Self { field, object_defn }
    }
}

pub(crate) fn check_associations(defns: &[Definition]) -> Vec<PositionedComment> {
    let object_defns: Vec<_> = defns.iter().filter_map(ObjectDefn::new).collect();
    let object_defns_map: HashMap<_, _> = object_defns
        .iter()
        .map(|defn| defn.name)
        .zip(&object_defns)
        .collect();

    let fields_with_associations: Vec<_> = object_defns
        .iter()
        .flat_map(|defn| {
            defn.fields
                .iter()
                .map(move |f| FieldWithAssociation::new(f, &defn))
        })
        .filter(|f| extract_field_type_name(&object_defns_map, &f.field).is_some())
        .collect();

    let mut comments = Vec::new();
    comments.append(&mut check_belongs_to(&fields_with_associations));
    comments.append(&mut check_fields_for_association(
        &fields_with_associations,
        &object_defns_map,
    ));

    comments
}

fn check_belongs_to(fields_with_associations: &[FieldWithAssociation]) -> Vec<PositionedComment> {
    fields_with_associations
        .iter()
        .filter(|f| !f.field.directives.iter().any(|d| &d.name == "belongsTo"))
        .map(|f| {
            let message = r#"Missing "@belongsTo" directive"#;
            let comment = Comment::new(Severity::Error, message.to_string());
            PositionedComment::new(f.field.position, comment)
        })
        .collect()
}

fn check_fields_for_association<'a>(
    fields_with_associations: &[FieldWithAssociation],
    object_defns: &HashMap<&String, &'a ObjectDefn>,
) -> Vec<PositionedComment> {
    fields_with_associations
        .iter()
        .filter_map(|f| {
            extract_field_type_name(object_defns, &f.field)
                .and_then(|f_type_name| object_defns.get(f_type_name).map(|defn| (f, *defn)))
        })
        .filter_map(|(f, object_defn)| {
            let plural_field_name = if f.object_defn.name.ends_with('s') {
                (f.object_defn.name.clone() + "es").to_mixed_case()
            } else {
                (f.object_defn.name.clone() + "s").to_mixed_case()
            };

            if !object_defn
                .fields
                .iter()
                .any(|f| f.name == plural_field_name)
            {
                let message = format!(
                    r#"Missing field "{}", due to association on object type {} - {}\n"#,
                    plural_field_name, f.object_defn.name, f.object_defn.position
                );
                let comment = Comment::new(Severity::Error, message.to_string());
                Some(PositionedComment::new(*object_defn.position, comment))
            } else {
                None
            }
        })
        .collect()
}

fn extract_field_type_name<'a>(
    object_defns_map: &HashMap<&String, &'a ObjectDefn>,
    f: &'a Field,
) -> Option<&'a String> {
    match f.field_type {
        Type::NamedType(ref type_name) if object_defns_map.contains_key(type_name) => {
            Some(type_name)
        }
        Type::NonNullType(ref inner_type) => match **inner_type {
            Type::NamedType(ref type_name) if object_defns_map.contains_key(type_name) => {
                Some(type_name)
            }
            _ => None,
        },
        _ => None,
    }
}
