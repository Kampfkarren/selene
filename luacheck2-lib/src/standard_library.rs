use std::collections::HashMap;

use serde::{de::{self, Deserializer}, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct StandardLibrary {
    #[serde(flatten)]
    globals: HashMap<String, Field>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    name: String,
    field_type: FieldType,
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let field_raw = FieldSerde::deserialize(&deserializer)?;

        if !field_raw.property && field_raw.args.is_none() && field_raw.children.is_empty() {
            return Err(de::Error::custom("can't determine what kind of field this is"));
        }

        if field_raw.property && field_raw.args.is_some() {
            return Err(de::Error::custom("field is both a property and a function"));
        }
    }
}

#[derive(Deserialize)]
struct FieldSerde {
    #[serde(default)]
    property: bool,
    #[serde(default)]
    args: Option<HashMap<String, Argument>>,
    #[serde(flatten)]
    children: HashMap<String, Field>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldType {
    Function(Vec<Argument>),
    Property,
    Table(Vec<Field>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Argument {
    required: Required,
    argument_type: ArgumentType,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArgumentType {
    Number,
    #[serde(rename = "...")]
    Vararg,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Required {
    NotRequired,
    Required(Option<String>),
}
