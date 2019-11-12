use combine::stream::state::SourcePosition;
use std::collections::HashMap;

pub type Name = String;

/// https://github.com/graphql/graphql-spec/blob/master/spec/Appendix%20B%20--%20Grammar%20Summary.md#document
pub struct Document {
    pub definitions: Vec<Definition>,
}

pub enum Definition {
    ObjectType(ObjectType),
    EnumType(EnumType),
}

pub struct ObjectType {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub fields: Vec<Field>,
}

pub struct EnumType {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Name,
    pub values: Vec<EnumValue>,
}

pub struct Directive {
    pub name: Name,
    pub arguments: Vec<DirectiveArgs>,
}

pub struct DirectiveArgs {
    pub name: Name,
    pub value: Value,
}

pub struct Field {
    pub position: Pos,
    pub description: Option<String>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub field_type: FieldType,
}

pub enum FieldType {
    NamedType(Name),
    NonNullType(Box<FieldType>),
    ListType(Box<FieldType>),
}

pub enum Value {
    Variable(Name),
    Int(i32),
    Float(f32),
    String(String),
    Boolean(bool),
    Null,
    Enum(Name),
    List(Vec<Value>),
    Object(HashMap<Name, Value>),
}

pub struct EnumValue {
    pub name: Name,
    pub description: Option<String>,
    pub position: Pos,
    pub directives: Vec<Directive>,
}

impl EnumValue {
    pub fn new(
        name: Name,
        description: Option<String>,
        position: Pos,
        directives: Vec<Directive>,
    ) -> Option<Self> {
        match name.as_str() {
            "true" | "false" | "null" => None,
            _ => Some(Self {
                name,
                description,
                position,
                directives,
            }),
        }
    }
}

pub struct Pos {
    pub line: i32,
    pub column: i32,
}

impl From<SourcePosition> for Pos {
    fn from(pos: SourcePosition) -> Self {
        Self {
            line: pos.line,
            column: pos.column,
        }
    }
}
