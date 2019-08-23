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

pub fn check(schema: &str) -> Result<()> {
    let document =
        graphql_parser::parse_schema(schema).map_err(|_| "could not parse schema".to_string())?;
    for defn in document.definitions {
        has_id(defn);
    }

    Ok(())
}

fn has_id(defn: Definition) {
    if let Definition::TypeDefinition(TypeDefinition::Object(ObjectType {
        fields,
        name,
        position,
        ..
    })) = defn
    {
        let (id_fields, other_fields): (Vec<_>, Vec<_>) =
            fields.iter().partition(|f| f.name == "id");
        check_id_fields(position, &name, &id_fields);
        check_fields_for_id(&other_fields);
    }
}

fn check_id_fields(position: Pos, name: &str, id_fields: &[&Field]) {
    match id_fields.len() {
        0 => {
            println!(
                "{} Missing id field on object type {}, consider adding one",
                position, name
            );
        }
        1 => {
            let id_field = id_fields.first().unwrap();
            let print_suggestion = || {
                println!(
                    r#"{} - Consider making this "{}: ID!""#,
                    id_field.position, id_field.name
                );
            };
            match id_field.field_type {
                Type::NamedType(ref type_name) if type_name == "ID" => print_suggestion(),
                Type::ListType(_) => print_suggestion(),
                Type::NonNullType(ref inner_type) => {
                    if let Type::NamedType(ref id) = **inner_type {
                        if id != "ID" {
                            print_suggestion();
                        }
                    }
                }
                _ => print_suggestion(),
            }
        }
        _ => {
            println!(
                "{} multiple fields with the same name on object type {}",
                position, name
            );
        }
    }
}

fn check_fields_for_id(fields: &[&Field]) {
    for f in fields {
        let print_suggestion = || {
            println!(r#"{} - Consider making this "id: ID!""#, f.position);
        };
        match f.field_type {
            Type::NamedType(ref type_name) if type_name == "ID" => print_suggestion(),
            Type::NonNullType(ref inner_type) => {
                if let Type::NamedType(ref id) = **inner_type {
                    if id == "ID" {
                        print_suggestion();
                    }
                }
            }
            _ => (),
        }
    }
}
