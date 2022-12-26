use crate::{
    ast_util::{
        first_code,
        visit_nodes::{NodeVisitor, VisitorType},
    },
    lint_exists,
    lints::{Diagnostic, Label, Severity},
    CheckerDiagnostic, LintVariation,
};
use full_moon::{ast::Ast, node::Node, tokenizer::TokenType};
use std::collections::HashSet;

const GLOBAL_LINT_PREFIX: &str = "#";

lazy_static::lazy_static! {
    static ref NODES_TO_IGNORE: HashSet<VisitorType> = {
        let mut set = HashSet::new();
        set.insert(VisitorType::VisitBlock);
        set
    };
}

#[derive(Clone, Debug)]
pub struct FilterConfiguration {
    global: bool,
    pub lint: String,
    variation: LintVariation,
}

#[derive(Clone, Debug)]
struct Filter {
    configuration: FilterConfiguration,
    comment_range: (usize, usize),
    range: (usize, usize),
}

#[derive(Default)]
struct FilterVisitor {
    comments_checked: HashSet<(usize, usize)>,
    ranges: Vec<Result<Filter, Diagnostic>>,
}

pub fn parse_comment(comment_original: &str) -> Option<Vec<FilterConfiguration>> {
    let comment = comment_original.split_whitespace().collect::<String>();

    let global_stripped = comment.strip_prefix(GLOBAL_LINT_PREFIX);
    let global = global_stripped.is_some();
    let config = global_stripped
        .unwrap_or(&comment)
        .strip_prefix("selene:")?;

    let mut variation = String::new();
    let mut lint = String::new();

    let mut check_lint = false;
    let mut finished = false;

    for character in config.chars() {
        if character == '(' {
            check_lint = true;
        } else if character == ')' {
            finished = true;
            break;
        } else if check_lint {
            lint.push(character);
        } else {
            variation.push(character);
        }
    }

    if !finished || variation.is_empty() || lint.is_empty() {
        return None;
    }

    let variation = match variation.as_str() {
        "allow" => LintVariation::Allow,
        "deny" => LintVariation::Deny,
        "warn" => LintVariation::Warn,
        _ => return None,
    };

    Some(
        lint.split(',')
            .map(|lint| FilterConfiguration {
                global,
                lint: lint.to_owned(),
                variation,
            })
            .collect(),
    )
}

impl NodeVisitor for FilterVisitor {
    fn visit_node(&mut self, node: &dyn Node, visitor_type: VisitorType) {
        if NODES_TO_IGNORE.contains(&visitor_type) {
            return;
        }

        let leading_trivia = node.surrounding_trivia().0;
        for trivia in leading_trivia {
            let (trivia_start_position, trivia_end_position) =
                (trivia.start_position(), trivia.end_position());
            let hash = (trivia_start_position.bytes(), trivia_end_position.bytes());

            if self.comments_checked.contains(&hash) {
                continue;
            }

            self.comments_checked.insert(hash);

            for comment in match trivia.token_type() {
                TokenType::SingleLineComment { comment } => comment,
                TokenType::MultiLineComment { comment, .. } => comment,
                _ => continue,
            }
            .lines()
            {
                let configurations = match parse_comment(comment) {
                    Some(configurations) => configurations,
                    None => continue,
                };

                let range = node.range().unwrap_or_else(|| {
                    panic!(
                        "node has no range (lint filter at L{}:{} - L{}:{}",
                        trivia_start_position.line(),
                        trivia_start_position.character(),
                        trivia_end_position.line(),
                        trivia_end_position.character()
                    )
                });

                self.ranges
                    .extend(configurations.into_iter().map(|configuration| {
                        if lint_exists(&configuration.lint) {
                            Ok(Filter {
                                configuration,
                                comment_range: (
                                    trivia.start_position().bytes(),
                                    trivia.end_position().bytes(),
                                ),
                                range: (range.0.bytes(), range.1.bytes()),
                            })
                        } else {
                            Err(Diagnostic::new(
                                "invalid_lint_filter",
                                format!("no lint named `{}` exists", configuration.lint),
                                Label::new((
                                    trivia_start_position.bytes(),
                                    trivia_end_position.bytes(),
                                )),
                            ))
                        }
                    }));
            }
        }
    }
}

fn get_filter_ranges(ast: &Ast) -> Vec<Result<Filter, Diagnostic>> {
    let mut filter_visitor = FilterVisitor::default();
    filter_visitor.visit_nodes(ast);
    filter_visitor.ranges
}

#[derive(Debug)]
enum FilterInstruction {
    Push {
        configuration: FilterConfiguration,
        bytes: usize,
    },

    Pop {
        bytes: usize,
    },
}

impl FilterInstruction {
    fn bytes(&self) -> usize {
        match self {
            FilterInstruction::Push { bytes, .. } => *bytes,
            FilterInstruction::Pop { bytes } => *bytes,
        }
    }
}

pub fn filter_diagnostics(
    ast: &Ast,
    mut diagnostics: Vec<CheckerDiagnostic>,
    invalid_lint_filter_severity: Severity,
) -> Vec<CheckerDiagnostic> {
    let filter_ranges = get_filter_ranges(ast);
    let (mut filters, mut failures) = (Vec::new(), Vec::new());
    let mut new_diagnostics;

    for thing in filter_ranges {
        match thing {
            Ok(filter) => filters.push(filter),
            Err(failure) => failures.push(failure),
        }
    }

    if filters.is_empty() {
        new_diagnostics = diagnostics;
    } else {
        // Filter ranges are translated into instructions for a stack
        let mut global_filters: Vec<Filter> = Vec::new();
        let mut instructions: Vec<FilterInstruction> = Vec::new();
        let mut conflicting: Option<((usize, usize), Vec<Filter>)> = None;
        let first_code = first_code(ast);

        for filter in filters {
            // Check for global filters
            if filter.configuration.global {
                if let Some(first_code) = first_code {
                    if filter.comment_range.0 >= first_code.0.bytes() {
                        failures.push(Diagnostic::new_complete(
                            "invalid_lint_filter",
                            "global filters must come before any code".to_owned(),
                            Label::new(filter.comment_range),
                            Vec::new(),
                            vec![Label::new_with_message(
                                (first_code.0.bytes(), first_code.1.bytes()),
                                "global filter must be before this".to_owned(),
                            )],
                        ));

                        continue;
                    }
                }
            }

            // Check for conflicting filters
            if let Some((range, ref mut filters)) = conflicting.as_mut() {
                if *range == filter.range {
                    for possibly_conflicting in filters.iter() {
                        if possibly_conflicting.configuration.lint == filter.configuration.lint {
                            failures.push(Diagnostic::new_complete(
                                "invalid_lint_filter",
                                "filter conflicts with a previous one for the same code".to_owned(),
                                Label::new(filter.comment_range),
                                Vec::new(),
                                vec![Label::new_with_message(
                                    possibly_conflicting.comment_range,
                                    "conflicts with this".to_owned(),
                                )],
                            ));
                        }
                    }

                    filters.push(filter.clone());
                } else {
                    conflicting = Some((filter.range, vec![filter.clone()]));
                }
            } else {
                conflicting = Some((filter.range, vec![filter.clone()]));
            }

            if filter.configuration.global {
                global_filters.push(filter);
            } else {
                instructions.insert(
                    instructions
                        .iter()
                        .position(|instruction| instruction.bytes() < filter.range.1)
                        .unwrap_or(instructions.len()),
                    FilterInstruction::Pop {
                        bytes: filter.range.1,
                    },
                );

                instructions.insert(
                    instructions
                        .iter()
                        .position(|instruction| instruction.bytes() < filter.range.0)
                        .unwrap_or(instructions.len()),
                    FilterInstruction::Push {
                        configuration: filter.configuration,
                        bytes: filter.range.0,
                    },
                );
            }
        }

        for global_filter in global_filters {
            instructions.push(FilterInstruction::Push {
                configuration: global_filter.configuration,
                bytes: 0,
            })
        }

        new_diagnostics = Vec::with_capacity(diagnostics.len());
        let mut stack = Vec::with_capacity(instructions.len());

        diagnostics.sort_by_key(|diagnostic| diagnostic.diagnostic.primary_label.range.0);

        'next_diagnostic: for diagnostic in diagnostics.into_iter() {
            let start_byte = diagnostic.diagnostic.primary_label.range.0 as usize;

            // Run all instructions from before this byte
            while let Some(instruction) = instructions.pop() {
                if instruction.bytes() <= start_byte {
                    match instruction {
                        FilterInstruction::Push { configuration, .. } => {
                            stack.push(configuration);
                        }

                        FilterInstruction::Pop { .. } => {
                            stack
                                .pop()
                                .expect("FilterInstruction::Pop instructed, but stack is empty");
                        }
                    }
                } else {
                    instructions.push(instruction);
                    break;
                }
            }

            // Find the most recent configuration for this lint, and respect it
            for configuration in stack.iter().rev() {
                if configuration.lint == diagnostic.diagnostic.code {
                    let severity = configuration.variation.to_severity();
                    if severity != Severity::Allow {
                        new_diagnostics.push(CheckerDiagnostic {
                            severity,
                            diagnostic: diagnostic.diagnostic,
                        });
                    }

                    continue 'next_diagnostic;
                }
            }

            // If no configuration touched this lint, pass it through identically
            new_diagnostics.push(diagnostic);
        }
    }

    new_diagnostics.extend(&mut failures.into_iter().map(|failure| CheckerDiagnostic {
        severity: invalid_lint_filter_severity,
        diagnostic: failure,
    }));

    new_diagnostics
}

#[cfg(test)]
mod tests {
    use crate::{
        test_util::{test_full_run, test_full_run_config},
        CheckerConfig, LintVariation,
    };
    use std::collections::HashMap;

    #[test]
    fn test_lint_filtering() {
        test_full_run("lint_filtering", "lint_filtering");
    }

    #[test]
    fn test_just_comments() {
        test_full_run("lint_filtering", "just_comments");
    }

    #[test]
    fn test_manual_table_clone() {
        test_full_run("lint_filtering", "manual_table_clone");
    }

    #[test]
    fn test_deny_allowed_in_config() {
        test_full_run_config(
            "lint_filtering",
            "deny_allowed_in_config",
            CheckerConfig {
                lints: {
                    let mut map = HashMap::new();
                    map.insert("unused_variable".to_owned(), LintVariation::Allow);
                    map
                },
                ..CheckerConfig::default()
            },
        );
    }
}
