use graphql_parser::{
    self,
    query::Type,
    schema::{Definition, ObjectType, TypeDefinition},
};
use std::error::Error;

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

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
        let mut id_fields: Vec<_> = fields.iter().filter(|f| f.name == "id").collect();
        match id_fields.len() {
            0 => {
                println!(
                    "{} Missing id field on object type {}, consider adding one",
                    position, name
                );
            }
            1 => {
                let id_field = id_fields.pop().unwrap();
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
}
