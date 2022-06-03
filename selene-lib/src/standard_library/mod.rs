mod v1;

use std::{
    collections::{BTreeMap, HashMap},
    fmt, fs, io,
    path::Path,
};

use serde::{
    de::{self, Deserializer, Visitor},
    ser::{SerializeMap, SerializeSeq, Serializer},
    Deserialize, Serialize,
};

lazy_static::lazy_static! {
    static ref ANY_TABLE: BTreeMap<String, Field> = {
        let mut map = BTreeMap::new();
        map.insert("*".to_owned(), Field {
            field_kind: FieldKind::Any,
        });
        map
    };
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StandardLibrary {
    #[serde(default)]
    pub base: Option<String>,

    #[serde(default)]
    pub name: Option<String>,

    pub globals: BTreeMap<String, Field>,

    #[serde(default)]
    pub structs: BTreeMap<String, BTreeMap<String, Field>>,
}

#[derive(Debug)]
pub enum StandardLibraryError {
    DeserializeError(toml::de::Error),
    IoError(io::Error),
}

impl fmt::Display for StandardLibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StandardLibraryError::DeserializeError(error) => {
                write!(formatter, "deserialize error: {}", error)
            }
            StandardLibraryError::IoError(error) => write!(formatter, "io error: {}", error),
        }
    }
}

impl std::error::Error for StandardLibraryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use StandardLibraryError::*;

        match self {
            DeserializeError(error) => Some(error),
            IoError(error) => Some(error),
        }
    }
}

impl From<io::Error> for StandardLibraryError {
    fn from(error: io::Error) -> Self {
        StandardLibraryError::IoError(error)
    }
}

impl StandardLibrary {
    pub fn from_name(name: &str) -> Option<StandardLibrary> {
        macro_rules! names {
            {$($name:expr => $path:expr,)+} => {
                match name {
                    $(
                        $name => {
                            let mut std = toml::from_str::<StandardLibrary>(
                                include_str!($path)
                            ).unwrap_or_else(|error| {
                                panic!(
                                    "default standard library '{}' failed deserialization: {}",
                                    name,
                                    error,
                                )
                            });

                            if let Some(meta) = &std.meta {
                                if let Some(base_name) = &meta.base {
                                    let base = StandardLibrary::from_name(base_name);

                                    std.extend(
                                        base.expect("built-in library based off of non-existent built-in"),
                                    );
                                }
                            }

                            std.inflate();

                            Some(std)
                        },
                    )+

                    _ => None
                }
            };
        }

        // names! {
        //     "lua51" => "../../default_std/lua51.toml",
        //     "lua52" => "../../default_std/lua52.toml",
        // }
        todo!("convert the tomls to yml")
    }

    pub fn from_config_name(
        name: &str,
        directory: Option<&Path>,
    ) -> Result<Option<StandardLibrary>, StandardLibraryError> {
        let mut library: Option<StandardLibrary> = None;

        for segment in name.split('+') {
            let segment_library = match StandardLibrary::from_name(segment) {
                Some(default) => default,

                None => {
                    let mut path = directory
                        .map(Path::to_path_buf)
                        .unwrap_or_else(||
                            panic!(
                                "from_config_name used with no directory, but segment `{}` is not a built-in library",
                                segment
                            )
                        );

                    path.push(format!("{}.toml", segment));
                    match StandardLibrary::from_file(&path)? {
                        Some(library) => library,
                        None => return Ok(None),
                    }
                }
            };

            match library {
                Some(ref mut base) => base.extend(segment_library),
                None => library = Some(segment_library),
            };
        }

        if let Some(ref mut library) = library {
            library.inflate();
        }

        Ok(library)
    }

    pub fn from_file(filename: &Path) -> Result<Option<StandardLibrary>, StandardLibraryError> {
        let content = fs::read_to_string(filename)?;
        let mut library: StandardLibrary =
            toml::from_str(&content).map_err(StandardLibraryError::DeserializeError)?;

        if let Some(base_name) = &library.base {
            if let Some(base) = StandardLibrary::from_config_name(base_name, filename.parent())? {
                library.extend(base);
            }
        }

        Ok(Some(library))
    }

    // TODO: This can be significantly slimmed down to just joining "."
    pub fn find_global(&self, names: &[String]) -> Option<&Field> {
        todo!("StandardLibrary::find_global")
    }

    pub fn unstruct<'a>(&'a self, field: &'a Field) -> &'a Field {
        todo!("StandardLibrary::unstruct")
    }

    pub fn extend(&mut self, mut other: StandardLibrary) {
        todo!("StandardLibrary::extend")
    }

    pub fn inflate(&mut self) {
        todo!("StandardLibrary::inflate")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct FunctionBehavior {
    #[serde(rename = "args")]
    pub arguments: Vec<Argument>,
    pub method: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Field {
    #[serde(flatten)]
    pub field_kind: FieldKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldKind {
    Any,
    Function(FunctionBehavior),
    Property(PropertyWritability),
    Struct(String),
    Removed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TrueOnly;

impl<'de> Deserialize<'de> for TrueOnly {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match bool::deserialize(deserializer) {
            Ok(true) => Ok(TrueOnly),
            _ => Err(de::Error::custom("expected `true`")),
        }
    }
}

impl Serialize for TrueOnly {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(true)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
enum FieldKindSerde {
    Any { any: TrueOnly },
    Function(FunctionBehavior),
    Removed { removed: TrueOnly },
    Property { property: PropertyWritability },
    Struct { r#struct: String },
}

impl<'de> Deserialize<'de> for FieldKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let field_kind = FieldKindSerde::deserialize(deserializer)?;

        Ok(match field_kind {
            FieldKindSerde::Any { .. } => FieldKind::Any,
            FieldKindSerde::Function(function_behavior) => FieldKind::Function(function_behavior),
            FieldKindSerde::Removed { .. } => FieldKind::Removed,
            FieldKindSerde::Property { property } => FieldKind::Property(property),
            FieldKindSerde::Struct { r#struct } => FieldKind::Struct(r#struct),
        })
    }
}

impl Serialize for FieldKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_kind = match self {
            FieldKind::Any => FieldKindSerde::Any { any: TrueOnly },
            FieldKind::Function(function_behavior) => {
                FieldKindSerde::Function(function_behavior.to_owned())
            }
            FieldKind::Removed => FieldKindSerde::Removed { removed: TrueOnly },
            FieldKind::Property(property_writability) => FieldKindSerde::Property {
                property: *property_writability,
            },

            FieldKind::Struct(r#struct) => FieldKindSerde::Struct {
                r#struct: r#struct.to_owned(),
            },
        };

        field_kind.serialize(serializer)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PropertyWritability {
    ReadOnly,
    // New fields can be added and set, but variable itself cannot be redefined
    NewFields,
    // New fields can't be added, but entire variable can be overridden
    OverrideFields,
    // New fields can be added and entire variable can be overridden
    FullWrite,
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
// TODO: Nilable types
pub enum ArgumentType {
    Any,
    Bool,
    Constant(Vec<String>),
    Display(String),
    // TODO: Optionally specify parameters
    Function,
    Nil,
    Number,
    String,
    // TODO: Types for tables
    Table,
    // TODO: Support repeating types (like for string.char)
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
            other => Err(de::Error::custom(format!("unknown type {}", other))),
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
                    .map(|string| format!("\"{}\"", string))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ArgumentType::Display(display) => write!(formatter, "{}", display),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_serde() {
        StandardLibrary::from_name("lua51").expect("lua51.toml wasn't found");
        StandardLibrary::from_name("lua52").expect("lua52.toml wasn't found");
    }
}
