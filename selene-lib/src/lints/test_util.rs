use super::{Applicability, AstContext, Context, Diagnostic, Lint};
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

/// Returns empty string if there are no diffs
fn generate_diff(source1: &str, source2: &str, diagnostics: &[&Diagnostic]) -> String {
    let mut result = String::new();
    let mut has_changes = false;
    let mut byte_offset = 0;
    let mut prev_non_insert_applicability_prefix = "    ";

    for change in TextDiff::from_lines(source1, source2).iter_all_changes() {
        let change_length = change.value().len() as u32;

        let change_end_byte = byte_offset + change_length;

        let mut applicability_prefix = "    ";
        for diagnostic in diagnostics {
            let (start, end) = diagnostic.primary_label.range;
            if start < change_end_byte && end > byte_offset {
                if diagnostic.applicability == Applicability::MachineApplicable {
                    applicability_prefix = "[MA]";
                    break;
                } else if diagnostic.applicability == Applicability::MaybeIncorrect {
                    applicability_prefix = "[MI]";
                    break;
                }
            }
        }

        if change.tag() == ChangeTag::Insert {
            applicability_prefix = prev_non_insert_applicability_prefix;
        }

        let sign = match change.tag() {
            ChangeTag::Delete => {
                has_changes = true;
                format!("-{}", applicability_prefix)
            }
            ChangeTag::Insert => {
                has_changes = true;
                format!("+{}", applicability_prefix)
            }
            ChangeTag::Equal => "     ".to_string(),
        };

        result.push_str(&format!("{} {}", sign, change.value()));

        if change.tag() != ChangeTag::Insert {
            byte_offset = change_end_byte;
            prev_non_insert_applicability_prefix = applicability_prefix;
        }
    }

    if has_changes {
        result
    } else {
        "".to_string()
    }
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

    let context = Context {
        standard_library: config.standard_library,
        user_set_standard_library: if standard_library_is_set {
            Some(vec!["test-set".to_owned()])
        } else {
            None
        },
    };

    let mut diagnostics = lint.pass(&ast, &context, &AstContext::from_ast(&ast, &lua_source));
    diagnostics.sort_by_key(|diagnostic| diagnostic.primary_label.range);

    let mut fixed_code = lua_source.to_string();
    let mut fixed_diagnostics = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.fixed_code.is_some()
                && (diagnostic.applicability == Applicability::MachineApplicable
                    || diagnostic.applicability == Applicability::MaybeIncorrect)
        })
        .collect::<Vec<_>>();

    fixed_code = Diagnostic::get_applied_suggestions_code(
        fixed_code.as_str(),
        fixed_diagnostics,
        |new_code| {
            println!("Fixer generated code:\n{}", new_code);
            let fixed_ast = full_moon::parse(new_code).unwrap_or_else(|_| {
                panic!(
                    "Fixer generated invalid code:\n\
                        ----------------\n\
                        {}\n\
                        ----------------\n",
                    new_code
                )
            });
            lint.pass(
                &fixed_ast,
                &context,
                &AstContext::from_ast(&fixed_ast, &new_code.to_string()),
            )
        },
    );

    let fixed_ast = full_moon::parse(&fixed_code).unwrap_or_else(|_| {
        panic!(
            "Fixer generated invalid code:\n\
                ----------------\n\
                {}\n\
                ----------------\n",
            fixed_code
        )
    });
    let lint_results = lint.pass(
        &fixed_ast,
        &context,
        &AstContext::from_ast(&fixed_ast, &fixed_code),
    );
    fixed_diagnostics = lint_results.iter().collect::<Vec<_>>();
    fixed_diagnostics.sort_by_key(|diagnostic| diagnostic.start_position());

    let fixed_diff = generate_diff(
        &lua_source,
        &fixed_code,
        &diagnostics.iter().collect::<Vec<_>>(),
    );
    let diff_output_path = path_base.with_extension("fixed.diff");

    if let Ok(expected) = fs::read_to_string(&diff_output_path) {
        pretty_assertions::assert_eq!(PrettyString(&expected), PrettyString(&fixed_diff));
    } else {
        let mut output_file =
            fs::File::create(diff_output_path).expect("couldn't create fixed.diff output file");
        output_file
            .write_all(fixed_diff.as_bytes())
            .expect("couldn't write fixed.diff to output file.");
    }

    let mut files = codespan::Files::new();
    let source_id = files.add(format!("{test_name}.lua"), lua_source.clone());

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
