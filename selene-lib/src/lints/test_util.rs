use super::{AstContext, Context, Diagnostic, Lint};
use crate::{
    test_util::{get_standard_library, PrettyString},
    StandardLibrary,
};
use similar::{ChangeTag, TextDiff};
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

fn replace_code_range(code: &str, start: usize, end: usize, replacement: &str) -> String {
    if start > end || end > code.len() {
        return code.to_string();
    }

    return format!("{}{}{}", &code[..start], replacement, &code[end..]);
}

// Assumes diagnostics is sorted by starting ranges and that there are no overlapping ranges
fn apply_diagnostics_fixes(code: &str, diagnostics: &Vec<Diagnostic>) -> String {
    let mut bytes_offset = 0;

    let new_code = diagnostics
        .iter()
        .fold(code.to_string(), |code, diagnostic| {
            if diagnostic.fixed_code.is_some() {
                let new_code = replace_code_range(
                    code.as_str(),
                    (diagnostic.primary_label.range.0 as isize + bytes_offset as isize) as usize,
                    (diagnostic.primary_label.range.1 as isize + bytes_offset as isize) as usize,
                    &diagnostic.fixed_code.clone().unwrap().as_str(),
                );

                bytes_offset += diagnostic.fixed_code.as_ref().unwrap().len() as isize
                    - (diagnostic.primary_label.range.1 - diagnostic.primary_label.range.0)
                        as isize;

                new_code
            } else {
                code
            }
        });

    full_moon::parse(&new_code).expect("Failed to parse fixed code");

    new_code
}

fn generate_diff(source1: &str, source2: &str) -> String {
    let mut result = String::new();

    for change in TextDiff::from_lines(source1, source2).iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        result.push_str(&format!("{}{}", sign, change.value()));
    }

    result
}

pub fn test_lint_config_with_output<
    C: DeserializeOwned,
    E: std::error::Error,
    R: Lint<Config = C, Error = E>,
>(
    lint: R,
    lint_name: &'static str,
    test_name: &'static str,
    mut config: TestUtilConfig,
    output_extension: &str,
) {
    let path_base = TEST_PROJECTS_ROOT.join(lint_name).join(test_name);

    let configured_standard_library = get_standard_library(&path_base);
    let standard_library_is_set =
        config.standard_library != StandardLibrary::from_name("lua51").unwrap();

    if let Some(standard_library) = configured_standard_library {
        config.standard_library = standard_library;
    }

    let lua_source =
        fs::read_to_string(path_base.with_extension("lua")).expect("Cannot find lua file");

    let ast = full_moon::parse(&lua_source).expect("Cannot parse lua file");
    let mut diagnostics = lint.pass(
        &ast,
        &Context {
            standard_library: config.standard_library,
            user_set_standard_library: if standard_library_is_set {
                Some(vec!["test-set".to_owned()])
            } else {
                None
            },
        },
        &AstContext::from_ast(&ast),
    );

    let mut files = codespan::Files::new();
    let source_id = files.add(format!("{test_name}.lua"), lua_source.clone());

    diagnostics.sort_by_key(|diagnostic| diagnostic.primary_label.range);

    let mut output = termcolor::NoColor::new(Vec::new());

    let fixed_lua_code = apply_diagnostics_fixes(&lua_source, &diagnostics);
    let fixed_diff = generate_diff(&lua_source, &fixed_lua_code);
    let diff_output_path = path_base.with_extension("fixed.diff");

    if let Ok(expected) = fs::read_to_string(&diff_output_path) {
        pretty_assertions::assert_eq!(PrettyString(&expected), PrettyString(&fixed_diff));
    } else {
        let mut output_file =
            fs::File::create(diff_output_path).expect("couldn't create fixed.diff output file");
        output_file
            .write_all(fixed_diff.as_bytes())
            .expect("couldn't write fixed.diff to output file");
    }

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

pub fn test_lint_config<
    C: DeserializeOwned,
    E: std::error::Error,
    R: Lint<Config = C, Error = E>,
>(
    lint: R,
    lint_name: &'static str,
    test_name: &'static str,
    config: TestUtilConfig,
) {
    test_lint_config_with_output(lint, lint_name, test_name, config, "stderr");
}

pub fn test_lint<C: DeserializeOwned, E: std::error::Error, R: Lint<Config = C, Error = E>>(
    lint: R,
    lint_name: &'static str,
    test_name: &'static str,
) {
    test_lint_config(lint, lint_name, test_name, TestUtilConfig::default());
}
