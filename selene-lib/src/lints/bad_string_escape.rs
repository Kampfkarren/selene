use super::*;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    tokenizer,
    visitors::Visitor,
};
use regex::Regex;

lazy_static::lazy_static! {
    static ref STRING_ESCAPE_REGEX: Regex = Regex::new(r"\\(u\{|.)([\da-fA-F]*)(\}?)").unwrap();
}

enum ReasonWhy {
    CodepointTooHigh,
    DecimalTooHigh,
    DoubleInSingle,
    Invalid,
    Malformed,
    SingleInDouble,
}

pub struct BadStringEscapeLint;

impl Lint for BadStringEscapeLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(BadStringEscapeLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = BadStringEscapeVisitor {
            sequences: Vec::new(),
            roblox: context.is_roblox(),
        };

        visitor.visit_ast(ast);

        visitor
            .sequences
            .iter()
            .map(|sequence| match sequence.issue {
                ReasonWhy::Invalid => Diagnostic::new(
                    "bad_string_escape",
                    "string escape sequence doesn't exist".to_owned(),
                    Label::new(sequence.range.to_owned()),
                ),
                ReasonWhy::Malformed => Diagnostic::new(
                    "bad_string_escape",
                    "string escape sequence is malformed".to_owned(),
                    Label::new(sequence.range.to_owned()),
                ),
                ReasonWhy::DecimalTooHigh => Diagnostic::new_complete(
                    "bad_string_escape",
                    "decimal escape is too high".to_owned(),
                    Label::new(sequence.range.to_owned()),
                    vec![
                        "help: the maximum codepoint allowed in decimal escapes is `255`"
                            .to_owned(),
                    ],
                    Vec::new(),
                ),
                ReasonWhy::CodepointTooHigh => Diagnostic::new_complete(
                    "bad_string_escape",
                    "unicode codepoint is too high for this escape sequence".to_owned(),
                    Label::new(sequence.range.to_owned()),
                    vec![
                        "help: the maximum codepoint allowed in unicode escapes is `10ffff`"
                            .to_owned(),
                    ],
                    Vec::new(),
                ),
                ReasonWhy::DoubleInSingle => Diagnostic::new(
                    "bad_string_escape",
                    "double quotes do not have to be escaped when inside single quoted strings"
                        .to_owned(),
                    Label::new(sequence.range.to_owned()),
                ),
                ReasonWhy::SingleInDouble => Diagnostic::new(
                    "bad_string_escape",
                    "single quotes do not have to be escaped when inside double quoted strings"
                        .to_owned(),
                    Label::new(sequence.range.to_owned()),
                ),
            })
            .collect()
    }
}

struct BadStringEscapeVisitor {
    sequences: Vec<StringEscapeSequence>,
    roblox: bool,
}

struct StringEscapeSequence {
    range: (usize, usize),
    issue: ReasonWhy,
}

impl Visitor for BadStringEscapeVisitor {
    fn visit_value(&mut self, node: &ast::Value) {
        if_chain::if_chain! {
            if let ast::Value::String(token) = node;
            if let tokenizer::TokenType::StringLiteral { literal, multi_line, quote_type } = token.token_type();
            if multi_line.is_none();
            then {
                let quote_type = *quote_type;
                let value_start = node.range().unwrap().0.bytes();

                for captures in STRING_ESCAPE_REGEX.captures_iter(literal) {
                    let start = value_start + captures.get(1).unwrap().start();

                    match &captures[1] {
                        "a" | "b" | "f" | "n" | "r" | "t" | "v" | "\\" => {},
                        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                            if captures[2].len() > 1 {
                                let hundreds = captures[1].parse::<u16>().unwrap_or(0) * 100;
                                let tens = captures[2][1..2].parse::<u16>().unwrap_or(0);
                                if hundreds + tens > 0xff {
                                    self.sequences.push(
                                        StringEscapeSequence{
                                            range: (start, start + 4),
                                            issue: ReasonWhy::DecimalTooHigh,
                                        }
                                    );
                                }
                            }
                        },
                        "\"" => {
                            if quote_type == tokenizer::StringLiteralQuoteType::Single {
                                self.sequences.push(
                                    StringEscapeSequence{
                                        range: (start, start + 2),
                                        issue: ReasonWhy::DoubleInSingle,
                                    }
                                );
                            }
                        },
                        "'" => {
                            if quote_type == tokenizer::StringLiteralQuoteType::Double {
                                self.sequences.push(
                                    StringEscapeSequence{
                                        range: (start, start + 2),
                                        issue: ReasonWhy::SingleInDouble,
                                    }
                                );
                            }
                        },
                        "z" => {
                            if !self.roblox {
                                self.sequences.push(
                                    StringEscapeSequence{
                                        range: (start, start + 2),
                                        issue: ReasonWhy::Invalid,
                                    }
                                );
                            }
                        },
                        "x" => {
                            if !self.roblox {
                                self.sequences.push(
                                    StringEscapeSequence{
                                        range: (start, start + 2),
                                        issue: ReasonWhy::Invalid,
                                    }
                                );
                                continue;
                            }
                            let second_capture_len = captures[2].len();
                            if second_capture_len != 2 {
                                self.sequences.push(
                                    StringEscapeSequence{
                                        range: (start, start + second_capture_len + 2),
                                        issue: ReasonWhy::Malformed
                                    }
                                );
                            }
                        },
                        "u{" => {
                            if !self.roblox {
                                self.sequences.push(
                                    StringEscapeSequence{
                                        range: (start, start + 2),
                                        issue: ReasonWhy::Invalid,
                                    }
                                );
                                continue;
                            }
                            let second_capture_len = captures[2].len();
                            if captures[3].is_empty() {
                                self.sequences.push(
                                    StringEscapeSequence{
                                        range: (start, start + second_capture_len + 3),
                                        issue: ReasonWhy::Malformed,
                                    }
                                );
                                continue;
                            }
                            let codepoint = u32::from_str_radix(&captures[2], 16).unwrap_or(0x0011_0000);
                            if codepoint > 0x0010_ffff {
                                self.sequences.push(
                                    StringEscapeSequence {
                                        range: (start, start + second_capture_len + 4),
                                        issue: ReasonWhy::CodepointTooHigh,
                                    }
                                );
                            }
                        },
                        _ => {
                            self.sequences.push(
                                StringEscapeSequence{
                                    range: (start, start + 2),
                                    issue: ReasonWhy::Invalid,
                                }
                            );
                        },
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_bad_string_escape() {
        test_lint(
            BadStringEscapeLint::new(()).unwrap(),
            "bad_string_escape",
            "lua51_string_escapes",
        );
    }

    #[test]
    #[cfg(feature = "roblox")]
    fn test_bad_string_escape_roblox() {
        test_lint(
            BadStringEscapeLint::new(()).unwrap(),
            "bad_string_escape",
            "roblox_string_escapes",
        );
    }
}
