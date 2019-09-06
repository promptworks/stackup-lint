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

pub(crate) fn extract_field_list_type_name(
    field_type: &Type,
    inside_list: bool,
) -> Option<&String> {
    match field_type {
        Type::NamedType(_) if !inside_list => None,
        Type::NamedType(name) => Some(name),
        Type::ListType(inner_type) => extract_field_list_type_name(inner_type, true),
        Type::NonNullType(inner_type) => extract_field_list_type_name(inner_type, inside_list),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_field_list_type_name() {
        let type_1 = Type::NamedType("String".to_string());
        let type_2 = Type::NonNullType(Box::new(type_1.clone()));
        let type_3 = Type::ListType(Box::new(type_1.clone()));
        let type_4 = Type::NonNullType(Box::new(type_3.clone()));
        let type_5 = Type::ListType(Box::new(Type::NonNullType(Box::new(Type::NamedType(
            "ID".to_string(),
        )))));
        let type_6 = Type::NonNullType(Box::new(type_5.clone()));

        assert!(extract_field_list_type_name(&type_1, false).is_none());
        assert!(extract_field_list_type_name(&type_2, false).is_none());
        assert_eq!(
            extract_field_list_type_name(&type_3, false),
            Some(&"String".to_string())
        );
        assert_eq!(
            extract_field_list_type_name(&type_4, false),
            Some(&"String".to_string())
        );
        assert_eq!(
            extract_field_list_type_name(&type_5, false),
            Some(&"ID".to_string())
        );
        assert_eq!(
            extract_field_list_type_name(&type_6, false),
            Some(&"ID".to_string())
        );
    }
}
