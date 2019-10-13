use std::{collections::HashMap, fmt};

use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct StandardLibrary {
    #[serde(flatten)]
    globals: HashMap<String, Field>,
}

impl StandardLibrary {
    pub fn from_name(name: &str) -> Option<StandardLibrary> {
        macro_rules! names {
            {$($name:expr => $path:expr,)+} => {
                match name {
                    $(
                        $name => Some(toml::from_str::<StandardLibrary>(
                            include_str!($path)
                        ).unwrap_or_else(|_| {
                            panic!(
                                "default standard library '{}' failed deserialization",
                                name
                            )
                        })),
                    )+

                    _ => None
                }
            };
        }

        names! {
            "lua51" => "../default_std/lua51.toml",
        }
    }

    pub fn find_global(&self, names: &[String]) -> Option<&Field> {
        assert!(!names.is_empty());
        let mut current = &self.globals;

        // Traverse through `foo.bar` in `foo.bar.baz`
        for name in names.iter().take(names.len() - 1) {
            if let Some(child) = current.get(name) {
                if let Field::Table(children) = child {
                    current = children;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        current.get(names.last().unwrap())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Field {
    Function(Vec<Argument>),
    Property,
    Table(HashMap<String, Field>),
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let field_raw = FieldSerde::deserialize(deserializer)?;

        if !field_raw.property && field_raw.args.is_none() && field_raw.children.is_empty() {
            return Err(de::Error::custom(
                "can't determine what kind of field this is",
            ));
        }

        if field_raw.property && field_raw.args.is_some() {
            return Err(de::Error::custom("field is both a property and a function"));
        }

        if field_raw.property {
            return Ok(Field::Property);
        }

        if let Some(args) = field_raw.args {
            // TODO: Don't allow vararg in the middle
            return Ok(Field::Function(args));
        }

        Ok(Field::Table(field_raw.children))
    }
}

#[derive(Deserialize)]
struct FieldSerde {
    #[serde(default)]
    property: bool,
    #[serde(default)]
    args: Option<Vec<Argument>>,
    #[serde(flatten)]
    children: HashMap<String, Field>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Argument {
    #[serde(default)]
    pub required: Required,
    #[serde(rename = "type")]
    pub argument_type: ArgumentType,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
// TODO: Nilable types
pub enum ArgumentType {
    Any,
    Bool,
    // TODO: Optionally specify parameters,
    Function,
    Nil,
    Number,
    String,
    // TODO: Types for tables
    Table,
    #[serde(rename = "...")]
    Vararg,
}

impl fmt::Display for ArgumentType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgumentType::Any => write!(formatter, "any"),
            ArgumentType::Bool => write!(formatter, "bool"),
            ArgumentType::Function => write!(formatter, "function"),
            ArgumentType::Nil => write!(formatter, "nil"),
            ArgumentType::Number => write!(formatter, "number"),
            ArgumentType::String => write!(formatter, "string"),
            ArgumentType::Table => write!(formatter, "table"),
            ArgumentType::Vararg => write!(formatter, "..."),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Required {
    NotRequired,
    Required(Option<String>),
}

impl Default for Required {
    fn default() -> Self {
        Required::Required(None)
    }
}

impl<'de> Deserialize<'de> for Required {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(RequiredVisitor)
    }
}

struct RequiredVisitor;

impl<'de> Visitor<'de> for RequiredVisitor {
    type Value = Required;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a boolean or a string message (when required)")
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        if value {
            Ok(Required::Required(None))
        } else {
            Ok(Required::NotRequired)
        }
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Required::Required(Some(value.to_owned())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_serde() {
        StandardLibrary::from_name("lua51").expect("lua51.toml wasn't found");
    }
}
