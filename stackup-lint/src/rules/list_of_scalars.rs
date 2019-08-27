use super::ObjectDefn;
use crate::interface::{Comment, PositionedComment, Severity};
use crate::SCALARS;
use graphql_parser::{
    self,
    schema::{Definition, Type},
};

pub(crate) fn check_for_list_of_scalars(defns: &[Definition]) -> Vec<PositionedComment> {
    defns
        .iter()
        .filter_map(ObjectDefn::new)
        .flat_map(|defn| {
            defn.fields
                .iter()
                .filter(|f| {
                    extract_field_list_type_name(&f.field_type, false)
                        .filter(|type_name| SCALARS.contains(*type_name))
                        .is_some()
                })
                .map(|f| {
                    let message =
                        r#"List of Scalars are not supported You may want an association instead"#;
                    let comment = Comment::new(Severity::Warning, message.to_string());
                    PositionedComment::new(f.position, comment)
                })
        })
        .collect()
}

fn extract_field_list_type_name(field_type: &Type, inside_list: bool) -> Option<&String> {
    match field_type {
        Type::NamedType(_) if !inside_list => None,
        Type::NamedType(name) => Some(name),
        Type::ListType(inner_type) => extract_field_list_type_name(inner_type, true),
        Type::NonNullType(inner_type) => extract_field_list_type_name(inner_type, inside_list),
    }
}
