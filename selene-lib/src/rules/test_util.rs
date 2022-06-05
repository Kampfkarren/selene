use super::{Context, Rule};
use crate::{standard_library::v1, test_util::PrettyString, StandardLibrary};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use codespan_reporting::{
    diagnostic::Severity as CodespanSeverity, term::Config as CodespanConfig,
};

use serde::de::DeserializeOwned;

lazy_static::lazy_static! {
    static ref TEST_PROJECTS_ROOT: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("lints");
}

pub struct TestUtilConfig {
    pub standard_library: StandardLibrary,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl Default for TestUtilConfig {
    fn default() -> Self {
        TestUtilConfig {
            standard_library: StandardLibrary::from_name("lua51").unwrap(),
            __non_exhaustive: (),
        }
    }
}

pub fn test_lint_config<
    C: DeserializeOwned,
    E: std::error::Error,
    R: Rule<Config = C, Error = E>,
>(
    rule: R,
    lint_name: &'static str,
    test_name: &'static str,
    mut config: TestUtilConfig,
) {
    let path_base = TEST_PROJECTS_ROOT.join(lint_name).join(test_name);

    if let Ok(test_std_contents) = fs::read_to_string(path_base.with_extension("std.toml")) {
        // config.standard_library = toml::from_str(&test_std_contents).unwrap();
        config.standard_library = toml::from_str::<v1::StandardLibrary>(&test_std_contents)
            .unwrap()
            .into();
    }

    let lua_source =
        fs::read_to_string(path_base.with_extension("lua")).expect("Cannot find lua file");

    let ast = full_moon::parse(&lua_source).expect("Cannot parse lua file");
    let mut diagnostics = rule.pass(
        &ast,
        &Context {
            standard_library: config.standard_library,
        },
    );

    let mut files = codespan::Files::new();
    let source_id = files.add(format!("{}.lua", test_name), lua_source);

    diagnostics.sort_by_key(|diagnostic| diagnostic.primary_label.range);

    let mut output = termcolor::NoColor::new(Vec::new());

    for diagnostic in diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.into_codespan_diagnostic(source_id, CodespanSeverity::Error))
    {
        codespan_reporting::term::emit(
            &mut output,
            &CodespanConfig::default(),
            &files,
            &diagnostic,
        )
        .expect("couldn't emit to codespan");
    }

    let stderr = std::str::from_utf8(output.get_ref()).expect("output not utf-8");
    let output_path = path_base.with_extension("stderr");

    if let Ok(expected) = fs::read_to_string(&output_path) {
        pretty_assertions::assert_eq!(PrettyString(&expected), PrettyString(stderr));
    } else {
        let mut output_file = fs::File::create(output_path).expect("couldn't create output file");
        output_file
            .write_all(output.get_ref())
            .expect("couldn't write to output file");
    }
}

pub fn test_lint<C: DeserializeOwned, E: std::error::Error, R: Rule<Config = C, Error = E>>(
    rule: R,
    lint_name: &'static str,
    test_name: &'static str,
) {
    test_lint_config(rule, lint_name, test_name, TestUtilConfig::default());
}
