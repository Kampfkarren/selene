use std::borrow::Borrow;

use crate::{standard_library::StandardLibrary, text};

pub fn possible_standard_library_notes<S: Borrow<str>>(
    name_path: &[S],
    user_set_standard_library: &Option<Vec<String>>,
) -> Vec<String> {
    let possible_standard_libraries = possible_standard_libraries(name_path);

    if possible_standard_libraries.is_empty() {
        return Vec::new();
    }

    let mut notes = vec![format!(
        "`{}` was found in the {} standard librar{}",
        name_path.join("."),
        text::english_list(&possible_standard_libraries),
        text::plural(possible_standard_libraries.len(), "y", "ies"),
    )];

    if let Some(user_set_standard_library) = user_set_standard_library {
        if possible_standard_libraries.iter().all(|library| {
            user_set_standard_library
                .iter()
                .any(|set_library| set_library == library)
        }) {
            let example = &possible_standard_libraries[0];

            notes.push(format!(
                "it looks like all of the possible standard libraries are set.\n\
                you may have an outdated copy in your root. try deleting it and running selene again.\n\
                for example, delete `{example}.yml` if you have it.",
            ));
        }
    } else {
        notes.push(format!(
            "you can set the standard library by putting the following inside selene.toml:\n{}",
            possible_standard_libraries
                .iter()
                .map(|library| format!("std = \"{library}\""))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    notes
}

fn possible_standard_libraries<S: Borrow<str>>(name_path: &[S]) -> Vec<&'static str> {
    assert!(!name_path.is_empty());

    let mut possible_standard_libraries = Vec::new();

    for (name, default_standard_library) in StandardLibrary::all_default_standard_libraries() {
        if default_standard_library.find_global(name_path).is_some() {
            possible_standard_libraries.push(*name);
        }
    }

    possible_standard_libraries.sort_unstable();

    #[cfg(feature = "roblox")]
    {
        static ROBLOX_BASE_STD: once_cell::sync::OnceCell<StandardLibrary> =
            once_cell::sync::OnceCell::new();

        let from_roblox_std = match name_path[0].borrow() {
            "game" | "plugin" | "script" | "workspace" => true,

            _ => {
                let roblox_std = ROBLOX_BASE_STD.get_or_init(StandardLibrary::roblox_base);

                roblox_std.find_global(name_path).is_some()
            }
        };

        if from_roblox_std {
            // Insert at the beginning because it's the most likely to be correct, given demographics
            possible_standard_libraries.insert(0, "roblox");
        }
    }

    possible_standard_libraries
}
