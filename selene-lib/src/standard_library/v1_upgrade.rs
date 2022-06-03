use std::collections::BTreeMap;

use super::{v1, *};

impl From<v1::StandardLibrary> for StandardLibrary {
    fn from(mut v1: v1::StandardLibrary) -> Self {
        let mut standard_library = StandardLibrary::default();

        // TODO: structs

        if let Some(meta) = v1.meta.take() {
            standard_library.base = meta.base;
            standard_library.name = meta.name;
        }

        let mut globals = BTreeMap::new();

        let mut v1_globals = v1.globals.into_iter().collect::<Vec<_>>();

        while let Some((name, field)) = v1_globals.pop() {
            match field {
                v1::Field::Any => {
                    globals.insert(
                        name,
                        Field {
                            field_kind: FieldKind::Any,
                        },
                    );
                }

                v1::Field::Complex { function, table } => {
                    for (child_name, child_field) in table {
                        v1_globals.push((format!("{name}.{child_name}").to_owned(), child_field));
                    }

                    if let Some(function) = function {
                        globals.insert(
                            name,
                            Field {
                                field_kind: FieldKind::Function(FunctionBehavior {
                                    arguments: function
                                        .arguments
                                        .into_iter()
                                        .map(Into::into)
                                        .collect(),
                                    method: function.method,
                                }),
                            },
                        );
                    }
                }

                v1::Field::Property { writable } => {
                    globals.insert(
                        name,
                        Field {
                            field_kind: FieldKind::Property(writable.into()),
                        },
                    );
                }

                v1::Field::Struct(struct_name) => {
                    globals.insert(
                        name,
                        Field {
                            field_kind: FieldKind::Struct(struct_name),
                        },
                    );
                }

                v1::Field::Removed => {
                    globals.insert(
                        name,
                        Field {
                            field_kind: FieldKind::Removed,
                        },
                    );
                }
            }
        }

        standard_library.globals = globals;

        standard_library
    }
}

impl From<v1::Argument> for Argument {
    fn from(v1_argument: v1::Argument) -> Self {
        Argument {
            required: v1_argument.required.into(),
            argument_type: v1_argument.argument_type.into(),
        }
    }
}

impl From<v1::ArgumentType> for ArgumentType {
    fn from(v1_argument_type: v1::ArgumentType) -> Self {
        match v1_argument_type {
            v1::ArgumentType::Any => ArgumentType::Any,
            v1::ArgumentType::Bool => ArgumentType::Bool,
            v1::ArgumentType::Constant(constants) => ArgumentType::Constant(constants),
            v1::ArgumentType::Display(message) => ArgumentType::Display(message),
            v1::ArgumentType::Function => ArgumentType::Function,
            v1::ArgumentType::Nil => ArgumentType::Nil,
            v1::ArgumentType::Number => ArgumentType::Number,
            v1::ArgumentType::String => ArgumentType::String,
            v1::ArgumentType::Table => ArgumentType::Table,
            v1::ArgumentType::Vararg => ArgumentType::Vararg,
        }
    }
}

impl From<Option<v1::Writable>> for PropertyWritability {
    fn from(v1_writable: Option<v1::Writable>) -> Self {
        match v1_writable {
            Some(v1::Writable::Full) => PropertyWritability::FullWrite,
            Some(v1::Writable::NewFields) => PropertyWritability::NewFields,
            Some(v1::Writable::Overridden) => PropertyWritability::OverrideFields,
            None => PropertyWritability::ReadOnly,
        }
    }
}

impl From<v1::Required> for Required {
    fn from(v1_required: v1::Required) -> Self {
        match v1_required {
            v1::Required::NotRequired => Required::NotRequired,
            v1::Required::Required(message) => Required::Required(message),
        }
    }
}
