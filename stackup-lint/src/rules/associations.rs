use super::ObjectDefn;
use crate::interface::{Comment, PositionedComment, Severity};
use crate::rules::list_of_scalars::extract_field_list_type_name;
use graphql_parser::{
    self,
    query::Type,
    schema::{Definition, Field},
};
use heck::MixedCase;
use std::collections::HashMap;

struct FieldWithAssociation<'a> {
    field: &'a Field,
    field_type_name: String,
    object_defn: &'a ObjectDefn<'a>,
}

impl<'a> FieldWithAssociation<'a> {
    fn new(field: &'a Field, field_type_name: String, object_defn: &'a ObjectDefn) -> Self {
        Self {
            field,
            field_type_name,
            object_defn,
        }
    }
}

struct FieldWithListType<'a> {
    field: &'a Field,
    field_type_name: String,
    object_defn: &'a ObjectDefn<'a>,
}

impl<'a> FieldWithListType<'a> {
    fn new(field: &'a Field, field_type_name: String, object_defn: &'a ObjectDefn) -> Self {
        Self {
            field,
            field_type_name,
            object_defn,
        }
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
        .flat_map(|defn| defn.fields.iter().map(move |f| (f, defn)))
        .filter_map(|(f, defn)| {
            extract_field_type_name(&object_defns_map, &f)
                .map(|f_type_name| FieldWithAssociation::new(f, f_type_name.to_owned(), defn))
        })
        .collect();

    let fields_with_lists_of_object_types: Vec<_> = object_defns
        .iter()
        .flat_map(|defn| defn.fields.iter().map(move |f| (f, defn)))
        .filter_map(|(f, defn)| {
            extract_field_list_type_name(&f.field_type, false)
                .filter(|f_type_name| object_defns_map.contains_key(f_type_name))
                .map(|f_type_name| FieldWithListType::new(f, f_type_name.to_owned(), defn))
        })
        .collect();

    let mut comments = Vec::new();
    comments.append(&mut check_belongs_to(&fields_with_associations));
    comments.append(&mut check_fields_for_association(
        &fields_with_associations,
        &object_defns_map,
    ));
    comments.append(&mut check_field_name_against_type_name(
        &fields_with_associations,
    ));
    comments.append(&mut check_list_of_object_types_without_association(
        &fields_with_associations,
        &fields_with_lists_of_object_types,
    ));

    comments
}

fn check_field_name_against_type_name(
    fields_with_associations: &[FieldWithAssociation],
) -> Vec<PositionedComment> {
    fields_with_associations
        .iter()
        .filter(|f| f.field_type_name.to_mixed_case() != f.field.name)
        .map(|f| {
            let message = format!(
                r#"Field name should be "{}""#,
                f.field_type_name.to_mixed_case()
            );
            let comment = Comment::new(Severity::Error, message.to_string());
            PositionedComment::new(f.field.position, comment)
        })
        .collect()
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

fn check_list_of_object_types_without_association(
    fields_with_associations: &[FieldWithAssociation],
    fields_with_lists_of_object_types: &[FieldWithListType],
) -> Vec<PositionedComment> {
    fields_with_lists_of_object_types
        .iter()
        .filter(|&f_list| {
            !fields_with_associations
                .iter()
                .any(|f_assoc| *f_list.object_defn.name == f_assoc.field_type_name)
        })
        .map(|f_list| {
            let message = format!(
                r#"Missing an association on object type "{0}".
                Try adding a field with a "@belongsTo" directive on "{0}""#,
                f_list.field_type_name
            );
            let comment = Comment::new(Severity::Warning, message.to_string());
            PositionedComment::new(f_list.field.position, comment)
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
