use super::{super::StandardLibrary, Context, Rule};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use codespan_reporting::{
    diagnostic::Severity as CodespanSeverity, term::Config as CodespanConfig,
};

use full_moon::ast::owned::Owned;
use serde::de::DeserializeOwned;

lazy_static::lazy_static! {
    static ref TEST_PROJECTS_ROOT: PathBuf =
        { Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("lints") };
}

pub fn test_lint<C: DeserializeOwned, E: std::error::Error, R: Rule<Config = C, Error = E>>(
    rule: R,
    lint_name: &'static str,
    test_name: &'static str,
) {
    let path_base = TEST_PROJECTS_ROOT.join(lint_name).join(test_name);

    let lua_source =
        fs::read_to_string(path_base.with_extension("lua")).expect("Cannot find lua file");

    let ast = full_moon::parse(&lua_source)
        .expect("Cannot parse lua file")
        .owned();
    let mut diagnostics = rule.pass(
        &ast,
        &Context {
            standard_library: toml::from_str::<StandardLibrary>(include_str!(
                "../../../luacheck2/standards/lua51.toml"
            ))
            .unwrap(),
        },
    );

    let mut files = codespan::Files::new();
    let source_id = files.add(format!("{}.lua", test_name), lua_source);

    diagnostics.sort_by_key(|diagnostic| diagnostic.primary_label.position);

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
        pretty_assertions::assert_eq!(expected, stderr);
    } else {
        let mut output_file = fs::File::create(output_path).expect("couldn't create output file");
        output_file
            .write_all(output.get_ref())
            .expect("couldn't write to output file");
    }
}
