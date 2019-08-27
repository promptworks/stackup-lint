use graphql_parser::{
    schema::{Definition, Field, ObjectType, TypeDefinition},
    Pos,
};

pub mod associations;
pub mod id;
pub mod list_of_scalars;

/// This is a wrapper around the Definition
/// enum from graphql_parser
pub(crate) struct ObjectDefn<'a> {
    pub fields: &'a Vec<Field>,
    pub name: &'a String,
    pub position: &'a Pos,
}

impl<'a> ObjectDefn<'a> {
    /// Use the Smart constructor pattern to ensure that we can only
    /// create an ObjectDefn from values that are object types
    pub fn new(defn: &'a Definition) -> Option<Self> {
        match defn {
            Definition::TypeDefinition(TypeDefinition::Object(ObjectType {
                fields,
                name,
                position,
                ..
            })) => Some(Self {
                fields,
                name,
                position,
            }),
            _ => None,
        }
    }
}
