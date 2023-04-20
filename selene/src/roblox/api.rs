use std::{collections::HashMap, fmt};

use serde::{
    de::{self, Deserializer, IntoDeserializer, Visitor},
    Deserialize,
};

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiDump {
    pub classes: Vec<ApiClass>,
    pub enums: Vec<ApiEnum>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiClass {
    pub name: String,
    pub superclass: String,
    pub members: Vec<ApiMember>,
    #[serde(default)]
    pub tags: Vec<String>,
}

// TODO: DRY
#[derive(Deserialize)]
#[serde(tag = "MemberType")]
pub enum ApiMember {
    Callback {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Tags")]
        tags: Option<Vec<String>>,
    },

    Event {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Tags")]
        tags: Option<Vec<String>>,
    },

    Function {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Tags")]
        tags: Option<Vec<String>>,
        #[serde(rename = "Parameters")]
        parameters: Vec<ApiParameter>,
    },

    Property {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "Tags")]
        tags: Option<Vec<String>>,
        #[serde(rename = "Security")]
        security: ApiPropertySecurity,
        #[serde(rename = "ValueType")]
        value_type: ApiValueType,
    },

    #[serde(other)]
    Unknown,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiParameter {
    pub default: Option<String>,
    #[serde(rename = "Type")]
    pub parameter_type: ApiValueType,
}

#[derive(Debug)]
pub enum ApiValueType {
    Class { name: String },
    DataType { value: ApiDataType },
    Group { value: ApiGroupType },
    Primitive { value: ApiPrimitiveType },
    Other { name: String },
}

impl<'de> Deserialize<'de> for ApiValueType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ApiValueTypeVisitor)
    }
}

struct ApiValueTypeVisitor;

impl<'de> Visitor<'de> for ApiValueTypeVisitor {
    type Value = ApiValueType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an api value type")
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut map: HashMap<String, String> = HashMap::new();

        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        let category = map.remove("Category").ok_or_else(|| {
            serde::de::Error::custom("api value type did not contain a `Category`")
        })?;

        let name = map
            .remove("Name")
            .ok_or_else(|| serde::de::Error::custom("api value type did not contain a `Name`"))?;

        Ok(match category.as_str() {
            "Class" => ApiValueType::Class { name },

            "DataType" => ApiValueType::DataType {
                value: ApiDataType::deserialize(name.into_deserializer())?,
            },

            "Group" => ApiValueType::Group {
                value: ApiGroupType::deserialize(name.into_deserializer())?,
            },

            "Primitive" => ApiValueType::Primitive {
                value: ApiPrimitiveType::deserialize(name.into_deserializer())?,
            },

            _ => ApiValueType::Other { name },
        })
    }
}

#[derive(Debug, Deserialize)]
pub enum ApiGroupType {
    #[serde(alias = "Array")]
    #[serde(alias = "Dictionary")]
    #[serde(alias = "Map")]
    Table,
    Tuple,
    Variant,

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiPrimitiveType {
    Bool,
    Double,
    Float,
    Int,
    Int64,
    String,

    #[serde(other)]
    Unknown,
}

#[derive(Debug)]
pub enum ApiDataType {
    CFrame,
    Content,
    Vector2,
    Vector3,
    UDim,
    UDim2,

    Other(String),
}

impl ApiDataType {
    // Ideally, we'd be creating typed structures for all of these, but they
    // currently only exist on globals like `workspace`, and so it's not
    // worth the effort to keep them always up to date with such low surface area.
    pub fn has_custom_methods(&self) -> bool {
        matches!(
            self,
            &ApiDataType::CFrame
                | &ApiDataType::UDim
                | &ApiDataType::UDim2
                | &ApiDataType::Vector2
                | &ApiDataType::Vector3
        )
    }
}

// Tagged enums do not support #[serde(other)]
impl<'de> Deserialize<'de> for ApiDataType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let string = String::deserialize(deserializer)?;

        Ok(match string.as_str() {
            "CFrame" => ApiDataType::CFrame,
            "Content" => ApiDataType::Content,
            "UDim" => ApiDataType::UDim,
            "UDim2" => ApiDataType::UDim2,
            "Vector2" => ApiDataType::Vector2,
            "Vector3" => ApiDataType::Vector3,
            _ => ApiDataType::Other(string),
        })
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiPropertySecurity {
    read: ApiPropertySecurityContext,
    write: ApiPropertySecurityContext,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub enum ApiPropertySecurityContext {
    #[default]
    None,
    #[serde(other)]
    Secure,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiEnum {
    pub items: Vec<ApiEnumItem>,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiEnumItem {
    pub name: String,

    #[serde(default)]
    pub legacy_names: Vec<String>,
}
