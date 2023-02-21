pub mod v1;
mod v1_upgrade;

use std::{
    borrow::{Borrow, Cow},
    collections::{BTreeMap, HashMap},
    fmt, io,
};

use once_cell::sync::OnceCell;
use regex::{Captures, Regex};
use serde::{
    de::{self, Deserializer, Visitor},
    ser::{SerializeMap, SerializeSeq, Serializer},
    Deserialize, Serialize,
};

lazy_static::lazy_static! {
    static ref ANY_TABLE: BTreeMap<String, Field> = {
        let mut map = BTreeMap::new();
        map.insert("*".to_owned(), Field::from_field_kind(FieldKind::Any));
        map
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GlobalTreeField {
    Key(String),
    ReadOnlyField,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GlobalTreeNode {
    children: BTreeMap<String, GlobalTreeNode>,
    field: GlobalTreeField,
}

impl GlobalTreeNode {
    fn field<'a>(&self, names_to_fields: &'a BTreeMap<String, Field>) -> &'a Field {
        static READ_ONLY_FIELD: Field =
            Field::from_field_kind(FieldKind::Property(PropertyWritability::ReadOnly));

        match &self.field {
            GlobalTreeField::Key(key) => names_to_fields
                .get(key)
                .unwrap_or_else(|| panic!("couldn't find {key} inside names_to_fields")),

            GlobalTreeField::ReadOnlyField => &READ_ONLY_FIELD,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct GlobalTreeCache {
    cache: BTreeMap<String, GlobalTreeNode>,

    #[cfg(debug_assertions)]
    last_globals_hash: u64,
}

#[profiling::function]
fn extract_into_tree(
    names_to_fields: &BTreeMap<String, Field>,
) -> BTreeMap<String, GlobalTreeNode> {
    let mut fields: BTreeMap<String, GlobalTreeNode> = BTreeMap::new();

    for name in names_to_fields.keys() {
        let mut current = &mut fields;

        let mut split = name.split('.').collect::<Vec<_>>();
        let final_name = split.pop().unwrap();

        for segment in split {
            current = &mut current
                .entry(segment.to_string())
                .or_insert_with(|| GlobalTreeNode {
                    field: GlobalTreeField::ReadOnlyField,
                    children: BTreeMap::new(),
                })
                .children;
        }

        let tree_field_key = GlobalTreeField::Key(name.to_owned());

        if let Some(existing_segment) = current.get_mut(final_name) {
            existing_segment.field = tree_field_key;
        } else {
            current.insert(
                final_name.to_string(),
                GlobalTreeNode {
                    field: tree_field_key,
                    children: BTreeMap::new(),
                },
            );
        }
    }

    fields
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StandardLibrary {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub globals: BTreeMap<String, Field>,

    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub structs: BTreeMap<String, BTreeMap<String, Field>>,

    /// Internal, used for the Roblox standard library
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<i64>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_selene_version: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub roblox_classes: BTreeMap<String, RobloxClass>,

    #[serde(skip)]
    global_tree_cache: OnceCell<GlobalTreeCache>,
}

#[derive(Debug)]
pub enum StandardLibraryError {
    DeserializeTomlError(toml::de::Error),
    DeserializeYamlError(serde_yaml::Error),
    IoError(io::Error),
}

impl fmt::Display for StandardLibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StandardLibraryError::DeserializeTomlError(error) => {
                write!(formatter, "deserialize toml error: {error}")
            }

            StandardLibraryError::DeserializeYamlError(error) => {
                write!(formatter, "deserialize yaml error: {error}")
            }

            StandardLibraryError::IoError(error) => write!(formatter, "io error: {error}"),
        }
    }
}

impl std::error::Error for StandardLibraryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use StandardLibraryError::*;

        match self {
            DeserializeTomlError(error) => Some(error),
            DeserializeYamlError(error) => Some(error),
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
    // This assumes globals has not changed, which it shouldn't by the time this is being used.
    fn global_tree_cache(&self) -> &BTreeMap<String, GlobalTreeNode> {
        // O(n) debug check to make sure globals doesn't change
        #[cfg(debug_assertions)]
        let hash = {
            use std::{
                collections::hash_map::DefaultHasher,
                hash::{Hash, Hasher},
            };

            profiling::scope!("global_tree_cache: hash");

            let mut hasher = DefaultHasher::new();
            self.globals.hash(&mut hasher);
            hasher.finish()
        };

        if let Some(cache) = self.global_tree_cache.get() {
            profiling::scope!("global_tree_cache: cache hit");

            #[cfg(debug_assertions)]
            assert_eq!(
                cache.last_globals_hash, hash,
                "globals changed after global_tree_cache has already been created"
            );

            return &cache.cache;
        }

        profiling::scope!("global_tree_cache: cache not set");

        &self
            .global_tree_cache
            .get_or_init(|| {
                profiling::scope!("global_tree_cache: create cache");
                GlobalTreeCache {
                    cache: extract_into_tree(&self.globals),

                    #[cfg(debug_assertions)]
                    last_globals_hash: hash,
                }
            })
            .cache
    }

    /// Find a global in the standard library through its name path.
    /// Handles all of the following cases:
    /// 1. "x.y" where `x.y` is explicitly defined
    /// 2. "x.y" where `x.*` is defined
    /// 3. "x.y" where `x` is a struct with a `y` or `*` field
    /// 4. "x.y.z" where `x.*.z` or `x.*.*` is defined
    /// 5. "x.y.z" where `x.y` or `x.*` is defined as "any"
    /// 6. "x.y" resolving to a read only property if only "x.y.z" (or x.y.*) is explicitly defined
    #[profiling::function]
    pub fn find_global<S: Borrow<str>>(&self, names: &[S]) -> Option<&Field> {
        assert!(!names.is_empty());

        if let Some(explicit_global) = self.globals.get(&names.join(".")) {
            profiling::scope!("find_global: explicit global");
            return Some(explicit_global);
        }

        // TODO: This is really stupid lol
        let mut last_extracted_struct;

        let mut current = self.global_tree_cache();
        let mut current_names_to_fields = &self.globals;

        profiling::scope!("find_global: look through global tree cache");

        for name in names.iter().take(names.len() - 1) {
            let found_segment = current.get(name.borrow()).or_else(|| current.get("*"))?;
            let field = found_segment.field(current_names_to_fields);

            match &field.field_kind {
                FieldKind::Any => {
                    return Some(field);
                }

                FieldKind::Struct(struct_name) => {
                    let strukt = self
                        .structs
                        .get(struct_name)
                        .unwrap_or_else(|| panic!("struct `{struct_name}` not found"));

                    last_extracted_struct = extract_into_tree(strukt);
                    current_names_to_fields = strukt;
                    current = &last_extracted_struct;
                }

                _ => {
                    current = &found_segment.children;
                }
            }
        }

        current
            .get(names.last().unwrap().borrow())
            .or_else(|| current.get("*"))
            .map(|node| node.field(current_names_to_fields))
    }

    pub fn global_has_fields(&self, name: &str) -> bool {
        profiling::scope!("global_has_fields", name);
        self.global_tree_cache().contains_key(name)
    }

    pub fn extend(&mut self, other: StandardLibrary) {
        self.structs.extend(other.structs);

        // let mut globals = other.globals.to_owned();
        let mut globals: BTreeMap<String, Field> = other
            .globals
            .into_iter()
            .filter(|(other_field_name, other_field)| {
                other_field.field_kind != FieldKind::Removed
                    && !matches!(
                        self.globals.get(other_field_name),
                        Some(Field {
                            field_kind: FieldKind::Removed,
                            ..
                        })
                    )
            })
            .collect();

        globals.extend(
            std::mem::take(&mut self.globals)
                .into_iter()
                .filter_map(|(key, value)| {
                    if value.field_kind == FieldKind::Removed {
                        None
                    } else {
                        Some((key, value))
                    }
                }),
        );

        self.globals = globals;
    }

    #[cfg(feature = "roblox")]
    pub fn roblox_base() -> StandardLibrary {
        StandardLibrary::from_builtin_name(
            "roblox",
            include_str!("../../default_std/roblox_base.yml"),
        )
        .expect("roblox_base.yml is missing")
    }

    fn from_builtin_name(name: &str, contents: &str) -> Option<StandardLibrary> {
        let mut std = serde_yaml::from_str::<StandardLibrary>(contents).unwrap_or_else(|error| {
            panic!("default standard library '{name}' failed deserialization: {error}")
        });

        if let Some(base_name) = &std.base {
            let base = StandardLibrary::from_name(base_name);

            std.extend(base.expect("built-in library based off of non-existent built-in"));
        }

        Some(std)
    }
}

macro_rules! names {
    {$($name:expr => $path:expr,)+} => {
        impl StandardLibrary {
            pub fn from_name(name: &str) -> Option<StandardLibrary> {
                match name {
                    $(
                        $name => {
                            StandardLibrary::from_builtin_name(name, include_str!($path))
                        },
                    )+

                    _ => None
                }
            }

            pub fn all_default_standard_libraries() -> &'static HashMap<&'static str, StandardLibrary> {
                static CACHED_RESULT: OnceCell<HashMap<&'static str, StandardLibrary>> = OnceCell::new();

                CACHED_RESULT.get_or_init(|| {
                    let mut stds = HashMap::new();

                    $(
                        stds.insert(
                            $name,
                            StandardLibrary::from_name($name).unwrap(),
                        );
                    )+

                    stds
                })
            }
        }
    };
}

names! {
    "lua51" => "../../default_std/lua51.yml",
    "lua52" => "../../default_std/lua52.yml",
    "lua53" => "../../default_std/lua53.yml",
    "luau" => "../../default_std/luau.yml",
}

fn is_default<T>(value: &T) -> bool
where
    T: Default + PartialEq<T>,
{
    value == &T::default()
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct FunctionBehavior {
    #[serde(rename = "args")]
    pub arguments: Vec<Argument>,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub method: bool,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub must_use: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct Field {
    #[serde(flatten)]
    pub field_kind: FieldKind,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Deprecated>,
}

impl Field {
    pub const fn from_field_kind(field_kind: FieldKind) -> Self {
        Self {
            field_kind,
            deprecated: None,
        }
    }

    pub fn with_deprecated(self, deprecated: Option<Deprecated>) -> Self {
        Self { deprecated, ..self }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Deprecated {
    pub message: String,

    // TODO: Validate proper %s
    // TODO: Validate that a pattern is possible to reach (uses different # of parameters)
    // TODO: Validate that parmeters match the number of arguments
    // TODO: Validate that all numbers parse as u32
    #[serde(default)]
    pub replace: Vec<String>,
}

impl Deprecated {
    fn regex_pattern() -> Regex {
        Regex::new(r"%(%|(?P<number>[0-9]+)|(\.\.\.))").unwrap()
    }

    pub fn try_instead(&self, parameters: &[String]) -> Option<String> {
        profiling::scope!("Deprecated::try_instead");

        let regex_pattern = Deprecated::regex_pattern();

        for replace_format in &self.replace {
            let mut success = true;

            let new_message = regex_pattern.replace_all(replace_format, |captures: &Captures| {
                if let Some(number) = captures.name("number") {
                    let number = match number.as_str().parse::<u32>() {
                        Ok(number) => number,
                        Err(_) => {
                            success = false;
                            return Cow::Borrowed("");
                        }
                    };

                    if number > parameters.len() as u32 || number == 0 {
                        success = false;
                        return Cow::Borrowed("");
                    }

                    return Cow::Borrowed(&parameters[number as usize - 1]);
                }

                let capture = captures.get(1).unwrap();
                match capture.as_str() {
                    "%" => Cow::Borrowed("%"),
                    "..." => Cow::Owned(parameters.join(", ")),
                    other => unreachable!("Unexpected capture in deprecated formatting: {}", other),
                }
            });

            if !success {
                continue;
            }

            return Some(new_message.into_owned());
        }

        None
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PropertyWritability {
    // New fields can't be added, and entire variable can't be overridden
    ReadOnly,
    // New fields can be added and set, but variable itself cannot be redefined
    NewFields,
    // New fields can't be added, but entire variable can be overridden
    OverrideFields,
    // New fields can be added and entire variable can be overridden
    FullWrite,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct Argument {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub required: Required,

    #[serde(rename = "type")]
    pub argument_type: ArgumentType,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub observes: Observes,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Observes {
    ReadWrite,
    Read,
    Write,
}

impl Default for Observes {
    fn default() -> Self {
        Self::ReadWrite
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct RobloxClass {
    pub superclass: String,
    pub events: Vec<String>,
    pub properties: Vec<String>,
}

impl RobloxClass {
    pub fn has_event(&self, roblox_classes: &BTreeMap<String, RobloxClass>, event: &str) -> bool {
        if self.events.iter().any(|other_event| other_event == event) {
            true
        } else if let Some(superclass) = roblox_classes.get(&self.superclass) {
            superclass.has_event(roblox_classes, event)
        } else {
            false
        }
    }

    pub fn has_property(
        &self,
        roblox_classes: &BTreeMap<String, RobloxClass>,
        property: &str,
    ) -> bool {
        if self
            .properties
            .iter()
            .any(|other_property| other_property == property)
        {
            true
        } else if let Some(superclass) = roblox_classes.get(&self.superclass) {
            superclass.has_property(roblox_classes, property)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string_vec(strings: Vec<&str>) -> Vec<String> {
        strings.into_iter().map(ToOwned::to_owned).collect()
    }

    #[test]
    fn valid_serde() {
        StandardLibrary::from_name("lua51").expect("lua51.toml wasn't found");
        StandardLibrary::from_name("lua52").expect("lua52.toml wasn't found");
    }

    #[test]
    fn deprecated_try_instead() {
        let deprecated = Deprecated {
            message: "You shouldn't see this".to_owned(),
            replace: vec![
                "eleven(%11)".to_owned(),
                "four(%1, %2, %3, %4)".to_owned(),
                "three(%1, %2, %3 %%3)".to_owned(),
                "two(%1, %2)".to_owned(),
                "one(%1)".to_owned(),
            ],
        };

        assert_eq!(
            deprecated.try_instead(&string_vec(vec!["a", "b", "c"])),
            Some("three(a, b, c %3)".to_owned())
        );

        assert_eq!(
            deprecated.try_instead(&string_vec(vec!["a", "b"])),
            Some("two(a, b)".to_owned())
        );

        assert_eq!(
            deprecated.try_instead(&string_vec(vec!["a"])),
            Some("one(a)".to_owned())
        );

        assert_eq!(
            deprecated.try_instead(&string_vec(vec![
                "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11",
            ])),
            Some("eleven(11)".to_owned())
        );

        assert_eq!(deprecated.try_instead(&string_vec(vec![])), None);
    }

    #[test]
    fn deprecated_varargs() {
        let deprecated = Deprecated {
            message: "You shouldn't see this".to_owned(),
            replace: vec!["print(%...)".to_owned()],
        };

        assert_eq!(
            deprecated.try_instead(&string_vec(vec!["a", "b", "c"])),
            Some("print(a, b, c)".to_owned())
        );
    }
}
