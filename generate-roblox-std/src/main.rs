use std::collections::BTreeMap;

use luacheck2_lib::standard_library::*;

mod api;

use api::*;

const API_DUMP: &str =
    "https://raw.githubusercontent.com/CloneTrooper1019/Roblox-Client-Tracker/roblox/API-Dump.json";

fn event_field() -> Field {
    let mut children = BTreeMap::new();

    children.insert(
        "Connect".to_owned(),
        Field::Function {
            arguments: vec![Argument {
                required: Required::Required(None),
                argument_type: ArgumentType::Function,
            }],
            method: true,
        },
    );

    children.insert(
        "Wait".to_owned(),
        Field::Function {
            arguments: Vec::new(),
            method: true,
        },
    );

    Field::Table(children)
}

fn write_class(std: &mut StandardLibrary, api: &api::ApiDump, global_name: &str, class_name: &str) {
    let class = api.classes.iter().find(|c| c.name == class_name).unwrap();
    let global_field = std
        .globals
        .entry(global_name.to_owned())
        .or_insert_with(|| Field::Table(BTreeMap::new()));

    let global_table = if let Field::Table(table) = global_field {
        table
    } else {
        unreachable!();
    };

    for member in &class.members {
        let (name, tags, field) = match &member {
            ApiMember::Callback { name, tags } => (
                name,
                tags,
                Some(Field::Property {
                    writable: Some(Writable::Overridden),
                }),
            ),

            ApiMember::Event { name, tags } => (name, tags, Some(event_field())),

            ApiMember::Function {
                name,
                tags,
                parameters,
            } => (
                name,
                tags,
                Some(Field::Function {
                    arguments: parameters
                        .iter()
                        .map(|param| Argument {
                            required: if param.default.is_some() {
                                Required::NotRequired
                            } else {
                                Required::Required(None)
                            },
                            argument_type: match &param.parameter_type {
                                ApiValueType::DataType { value } => match value {
                                    ApiDataType::Content => ArgumentType::String,
                                    ApiDataType::Other(other) => {
                                        ArgumentType::Display(other.to_owned())
                                    }
                                },

                                ApiValueType::Group { value } => match value {
                                    ApiGroupType::Table => ArgumentType::Table,
                                    ApiGroupType::Tuple => ArgumentType::Vararg,
                                    ApiGroupType::Variant => ArgumentType::Any,
                                },

                                ApiValueType::Primitive { value } => match value {
                                    ApiPrimitiveType::Bool => ArgumentType::Bool,
                                    ApiPrimitiveType::Double
                                    | ApiPrimitiveType::Float
                                    | ApiPrimitiveType::Int
                                    | ApiPrimitiveType::Int64 => ArgumentType::Number,
                                    ApiPrimitiveType::String => ArgumentType::String,
                                },

                                ApiValueType::Other { name } => {
                                    ArgumentType::Display(name.to_owned())
                                }
                            },
                        })
                        .collect(),
                    method: true,
                }),
            ),

            ApiMember::Property {
                name,
                tags,
                security,
            } => (name, tags, {
                if *security == ApiPropertySecurity::default() {
                    let empty = Vec::new();
                    let tags: &Vec<String> = match tags {
                        Some(tags) => tags,
                        None => &empty,
                    };

                    Some(Field::Property {
                        writable: if tags.contains(&"ReadOnly".to_string()) {
                            None
                        } else {
                            Some(Writable::Overridden)
                        },
                    })
                } else {
                    None
                }
            }),
        };

        let empty = Vec::new();
        let tags: &Vec<String> = match tags {
            Some(tags) => tags,
            None => &empty,
        };

        if tags.contains(&"Deprecated".to_owned()) {
            continue;
        }

        if let Some(field) = field {
            global_table.insert(name.to_owned(), field);
        }
    }

    if class.superclass != "<<<ROOT>>>" {
        write_class(std, api, global_name, &class.superclass);
    }
}

fn write_enums(std: &mut StandardLibrary, api: &api::ApiDump) {
    let mut children = BTreeMap::new();

    for enuhm in &api.enums {
        let mut enum_table = BTreeMap::new();

        for item in &enuhm.items {
            enum_table.insert(item.name.to_owned(), Field::Property { writable: None });
        }

        children.insert(enuhm.name.to_owned(), Field::Table(enum_table));
    }

    std.globals
        .insert("Enum".to_owned(), Field::Table(children));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut export: StandardLibrary = toml::from_str(include_str!("../base.toml"))?;

    let api: api::ApiDump = reqwest::get(API_DUMP)?.json()?;

    write_class(&mut export, &api, "game", "DataModel");
    write_class(&mut export, &api, "script", "Script");
    write_class(&mut export, &api, "workspace", "Workspace");

    write_enums(&mut export, &api);

    println!("# This file was @generated by generate-roblox-std");
    println!("{}", toml::to_string(&export).unwrap());

    Ok(())
}
