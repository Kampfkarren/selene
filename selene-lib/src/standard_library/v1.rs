// An intentionally separate implementation of the standard library as it existed
// before version 2.
// This is so that any changes to the v2 standard library don't affect the old one.
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};

use serde::{
    de::{self, Deserializer, Visitor},
    ser::{SerializeMap, SerializeSeq, Serializer},
    Deserialize, Serialize,
};

lazy_static::lazy_static! {
    static ref ANY_TABLE: BTreeMap<String, Field> = {
        let mut map = BTreeMap::new();
        map.insert("*".to_owned(), Field::Any);
        map
    };
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StandardLibrary {
    #[serde(rename = "selene")]
    pub meta: Option<StandardLibraryMeta>,
    #[serde(flatten)]
    pub globals: BTreeMap<String, Field>,
    #[serde(skip)]
    pub structs: BTreeMap<String, Field>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StandardLibraryMeta {
    #[serde(default)]
    pub base: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub structs: Option<BTreeMap<String, BTreeMap<String, Field>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionBehavior {
    pub arguments: Vec<Argument>,
    pub method: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Field {
    Any,
    Complex {
        function: Option<FunctionBehavior>,
        table: BTreeMap<String, Field>,
    },
    Property {
        writable: Option<Writable>,
    },
    Struct(String),
    Removed,
}

impl From<BTreeMap<String, Field>> for Field {
    fn from(table: BTreeMap<String, Field>) -> Self {
        Field::Complex {
            function: None,
            table,
        }
    }
}

impl From<FunctionBehavior> for Field {
    fn from(function: FunctionBehavior) -> Self {
        Field::Complex {
            function: Some(function),
            table: BTreeMap::new(),
        }
    }
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let field_raw = FieldSerde::deserialize(deserializer)?;

        if field_raw.any {
            return Ok(Field::Any);
        }

        if field_raw.removed {
            return Ok(Field::Removed);
        }

        let is_function = field_raw.args.is_some() || field_raw.method;

        if !field_raw.property
            && !is_function
            && field_raw.children.is_empty()
            && field_raw.strukt.is_none()
        {
            return Err(de::Error::custom(
                "can't determine what kind of field this is",
            ));
        }

        if field_raw.property && is_function {
            return Err(de::Error::custom("field is both a property and a function"));
        }

        if field_raw.property {
            return Ok(Field::Property {
                writable: field_raw.writable,
            });
        }

        if let Some(name) = field_raw.strukt {
            return Ok(Field::Struct(name));
        }

        let function_behavior = if is_function {
            Some(FunctionBehavior {
                arguments: field_raw.args.unwrap_or_default(),
                method: field_raw.method,
            })
        } else {
            None
        };

        Ok(Field::Complex {
            function: function_behavior,
            table: field_raw.children,
        })
    }
}

impl Serialize for Field {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Field::Any => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("any", &true)?;
                map.end()
            }

            Field::Complex { function, table } => {
                let mut map = serializer.serialize_map(None)?;

                if let Some(function) = function {
                    if function.method {
                        map.serialize_entry("method", &true)?;
                    }
                    map.serialize_entry("args", &function.arguments)?;
                }

                for (key, value) in table.iter() {
                    map.serialize_entry(key, value)?;
                }

                map.end()
            }

            Field::Property { writable } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("property", &true)?;
                if let Some(writable) = writable {
                    map.serialize_entry("writable", writable)?;
                }
                map.end()
            }

            Field::Struct(name) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("struct", name)?;
                map.end()
            }

            Field::Removed => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("removed", &true)?;
                map.end()
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Writable {
    // New fields can be added and set, but variable itself cannot be redefined
    NewFields,
    // New fields can't be added, but entire variable can be overridden
    Overridden,
    // New fields can be added and entire variable can be overridden
    Full,
}

#[derive(Debug, Deserialize)]
struct FieldSerde {
    #[serde(default)]
    property: bool,
    #[serde(default)]
    method: bool,
    #[serde(default)]
    removed: bool,
    #[serde(default)]
    writable: Option<Writable>,
    #[serde(default)]
    args: Option<Vec<Argument>>,
    #[serde(default)]
    #[serde(rename = "struct")]
    strukt: Option<String>,
    #[serde(default)]
    any: bool,
    #[serde(flatten)]
    children: BTreeMap<String, Field>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Argument {
    #[serde(default)]
    #[serde(skip_serializing_if = "Required::required_no_message")]
    pub required: Required,
    #[serde(rename = "type")]
    pub argument_type: ArgumentType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgumentType {
    Any,
    Bool,
    Constant(Vec<String>),
    Display(String),
    Function,
    Nil,
    Number,
    String,
    Table,
    Vararg,
}

impl Serialize for ArgumentType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            &ArgumentType::Any
            | &ArgumentType::Bool
            | &ArgumentType::Function
            | &ArgumentType::Nil
            | &ArgumentType::Number
            | &ArgumentType::String
            | &ArgumentType::Table
            | &ArgumentType::Vararg => serializer.serialize_str(&self.to_string()),

            ArgumentType::Constant(constants) => {
                let mut seq = serializer.serialize_seq(Some(constants.len()))?;
                for constant in constants {
                    seq.serialize_element(constant)?;
                }
                seq.end()
            }

            ArgumentType::Display(display) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("display", display)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ArgumentType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ArgumentTypeVisitor)
    }
}

struct ArgumentTypeVisitor;

impl<'de> Visitor<'de> for ArgumentTypeVisitor {
    type Value = ArgumentType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an argument type or an array of constant strings")
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut map: HashMap<String, String> = HashMap::new();

        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        if let Some(display) = map.remove("display") {
            Ok(ArgumentType::Display(display))
        } else {
            Err(de::Error::custom(
                "map value must have a `display` property",
            ))
        }
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut constants = Vec::new();

        while let Some(value) = seq.next_element()? {
            constants.push(value);
        }

        Ok(ArgumentType::Constant(constants))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        match value {
            "any" => Ok(ArgumentType::Any),
            "bool" => Ok(ArgumentType::Bool),
            "function" => Ok(ArgumentType::Function),
            "nil" => Ok(ArgumentType::Nil),
            "number" => Ok(ArgumentType::Number),
            "string" => Ok(ArgumentType::String),
            "table" => Ok(ArgumentType::Table),
            "..." => Ok(ArgumentType::Vararg),
            other => Err(de::Error::custom(format!("unknown type {other}"))),
        }
    }
}

impl fmt::Display for ArgumentType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgumentType::Any => write!(formatter, "any"),
            ArgumentType::Bool => write!(formatter, "bool"),
            ArgumentType::Constant(options) => write!(
                formatter,
                "{}",
                // TODO: This gets pretty ugly with a lot of variants
                options
                    .iter()
                    .map(|string| format!("\"{string}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ArgumentType::Display(display) => write!(formatter, "{display}"),
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

impl Required {
    fn required_no_message(&self) -> bool {
        self == &Required::Required(None)
    }
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

impl Serialize for Required {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Required::NotRequired => serializer.serialize_bool(false),
            Required::Required(None) => serializer.serialize_bool(true),
            Required::Required(Some(message)) => serializer.serialize_str(message),
        }
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
