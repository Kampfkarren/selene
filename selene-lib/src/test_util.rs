use crate::{standard_library::v1, Checker, CheckerConfig, Severity, StandardLibrary};
use std::{
    fmt, fs,
    io::Write,
    path::{Path, PathBuf},
};

use codespan_reporting::{
    diagnostic::Severity as CodespanSeverity, term::Config as CodespanConfig,
};

lazy_static::lazy_static! {
    static ref TEST_FULL_RUN_ROOT: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("full_run");
}

#[derive(PartialEq, Eq)]
#[doc(hidden)]
pub struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl<'a> fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

pub fn get_standard_library(path_base: &Path) -> Option<StandardLibrary> {
    if let Ok(test_std_toml_contents) = fs::read_to_string(path_base.with_extension("std.toml")) {
        Some(
            toml::from_str::<v1::StandardLibrary>(&test_std_toml_contents)
                .unwrap()
                .into(),
        )
    } else if let Ok(test_std_yml_contents) =
        fs::read_to_string(path_base.with_extension("std.yml"))
    {
        Some(serde_yaml::from_str(&test_std_yml_contents).unwrap())
    } else {
        None
    }
}

pub fn test_full_run_config_with_output(
    directory: &'static str,
    test_name: &'static str,
    checker_config: CheckerConfig<serde_json::Value>,
    output_extension: &str,
) {
    let path_base = TEST_FULL_RUN_ROOT.join(directory).join(test_name);

    let checker = Checker::<serde_json::Value>::new(
        checker_config,
        get_standard_library(&path_base).unwrap_or_else(|| {
            StandardLibrary::from_name("lua51").expect("no lua51 standard library")
        }),
    )
    .expect("couldn't create checker");

    let lua_source =
        fs::read_to_string(path_base.with_extension("lua")).expect("Cannot find lua file");

    let ast = full_moon::parse(&lua_source).expect("Cannot parse lua file");

    let mut diagnostics = checker.test_on(&ast);

    let mut files = codespan::Files::new();
    let source_id = files.add(format!("{test_name}.lua"), lua_source);

    diagnostics.sort_by_key(|diagnostic| diagnostic.diagnostic.primary_label.range);

    let mut output = termcolor::NoColor::new(Vec::new());

    for diagnostic in diagnostics.into_iter().filter_map(|diagnostic| {
        Some(diagnostic.diagnostic.into_codespan_diagnostic(
            source_id,
            match diagnostic.severity {
                Severity::Allow => return None,
                Severity::Error => CodespanSeverity::Error,
                Severity::Warning => CodespanSeverity::Warning,
            },
        ))
    }) {
        codespan_reporting::term::emit(
            &mut output,
            &CodespanConfig::default(),
            &files,
            &diagnostic,
        )
        .expect("couldn't emit to codespan");
    }

    let stderr = std::str::from_utf8(output.get_ref()).expect("output not utf-8");
    let output_path = path_base.with_extension(output_extension);

    if let Ok(expected) = fs::read_to_string(&output_path) {
        pretty_assertions::assert_eq!(PrettyString(&expected), PrettyString(stderr));
    } else {
        let mut output_file = fs::File::create(output_path).expect("couldn't create output file");
        output_file
            .write_all(output.get_ref())
            .expect("couldn't write to output file");
    }
}

// TODO: Most of this is copy and pasted from test_lint_config, try and abstract it out a bit
pub fn test_full_run_config(
    directory: &'static str,
    test_name: &'static str,
    checker_config: CheckerConfig<serde_json::Value>,
) {
    test_full_run_config_with_output(directory, test_name, checker_config, "stderr");
}

pub fn test_full_run(directory: &'static str, test_name: &'static str) {
    test_full_run_config(directory, test_name, CheckerConfig::default());
}
