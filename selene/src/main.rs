use std::{
    ffi::OsString,
    fmt, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

use codespan_reporting::{
    diagnostic::{
        Diagnostic as CodespanDiagnostic, Label as CodespanLabel, Severity as CodespanSeverity,
    },
    term::DisplayStyle as CodespanDisplayStyle,
};
use selene_lib::{rules::Severity, *};
use structopt::{clap, StructOpt};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use threadpool::ThreadPool;
use upgrade_std::upgrade_std;

#[cfg(feature = "roblox")]
use selene_lib::standard_library::StandardLibrary;

mod json_output;
mod opts;
#[cfg(feature = "roblox")]
mod roblox;
mod standard_library;
mod upgrade_std;

macro_rules! error {
    ($fmt:expr) => {
        error(&fmt::format(format_args!($fmt)))
    };

    ($fmt:expr, $($args:tt)*) => {
        error(&fmt::format(format_args!($fmt, $($args)*)))
    };
}

lazy_static::lazy_static! {
    static ref OPTIONS: RwLock<Option<opts::Options>> = RwLock::new(None);
}

static LINT_ERRORS: AtomicUsize = AtomicUsize::new(0);
static LINT_WARNINGS: AtomicUsize = AtomicUsize::new(0);
static PARSE_ERRORS: AtomicUsize = AtomicUsize::new(0);

fn get_color() -> ColorChoice {
    let lock = OPTIONS.read().unwrap();
    let opts = lock.as_ref().unwrap();

    match opts.color {
        opts::Color::Always => ColorChoice::Always,
        opts::Color::Auto => {
            if atty::is(atty::Stream::Stdout) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            }
        }
        opts::Color::Never => ColorChoice::Never,
    }
}

pub fn error(text: &str) {
    let mut stderr = StandardStream::stderr(get_color());
    stderr
        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
        .unwrap();
    write!(&mut stderr, "ERROR: ").unwrap();
    stderr.reset().unwrap();
    writeln!(&mut stderr, "{}", text).unwrap();
}

fn log_total(parse_errors: usize, lint_errors: usize, lint_warnings: usize) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(get_color());

    stdout.reset()?;
    writeln!(&mut stdout, "Results:")?;

    let mut stat = |number: usize, label: &str| -> io::Result<()> {
        if number > 0 {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        } else {
            stdout.reset()?;
        }

        write!(&mut stdout, "{}", number)?;
        stdout.reset()?;
        writeln!(&mut stdout, " {}", label)
    };

    stat(lint_errors, "errors")?;
    stat(lint_warnings, "warnings")?;
    stat(parse_errors, "parse errors")?;

    Ok(())
}

fn emit_codespan(
    writer: &mut impl termcolor::WriteColor,
    files: &codespan::Files<&str>,
    diagnostic: &CodespanDiagnostic<codespan::FileId>,
) {
    let lock = OPTIONS.read().unwrap();
    let opts = lock.as_ref().unwrap();

    let config = &codespan_reporting::term::Config {
        display_style: if opts.quiet() {
            CodespanDisplayStyle::Short
        } else {
            CodespanDisplayStyle::Rich
        },
        ..Default::default()
    };

    if let Some(opts::DisplayStyle::Json) = opts.display_style {
        writeln!(
            writer,
            "{}",
            json_output::diagnostic_to_json(diagnostic, files).unwrap()
        )
        .unwrap();
    } else {
        codespan_reporting::term::emit(writer, config, files, diagnostic)
            .expect("couldn't emit error to codespan");
    }
}

fn emit_codespan_locked(
    files: &codespan::Files<&str>,
    diagnostic: &CodespanDiagnostic<codespan::FileId>,
) {
    let stdout = termcolor::StandardStream::stdout(get_color());
    let mut stdout = stdout.lock();

    emit_codespan(&mut stdout, files, diagnostic);
}

fn read<R: Read>(checker: &Checker<toml::value::Value>, filename: &Path, mut reader: R) {
    let mut buffer = Vec::new();
    if let Err(error) = reader.read_to_end(&mut buffer) {
        error!(
            "Couldn't read contents of file {}: {}",
            filename.display(),
            error,
        );

        LINT_ERRORS.fetch_add(1, Ordering::SeqCst);
        return;
    }

    let contents = String::from_utf8_lossy(&buffer);

    let lock = OPTIONS.read().unwrap();
    let opts = lock.as_ref().unwrap();

    let mut files = codespan::Files::new();
    let source_id = files.add(filename.as_os_str(), &*contents);

    let ast = match full_moon::parse(&contents) {
        Ok(ast) => ast,
        Err(error) => {
            PARSE_ERRORS.fetch_add(1, Ordering::SeqCst);

            match error {
                full_moon::Error::AstError(full_moon::ast::AstError::UnexpectedToken {
                    token,
                    additional,
                }) => emit_codespan_locked(
                    &files,
                    &CodespanDiagnostic {
                        severity: CodespanSeverity::Error,
                        code: Some("parse_error".to_owned()),
                        message: format!("unexpected token `{}`", token),
                        labels: vec![CodespanLabel::primary(
                            source_id,
                            codespan::Span::new(
                                token.start_position().bytes() as u32,
                                token.end_position().bytes() as u32,
                            ),
                        )
                        .with_message(additional.unwrap_or_default())],
                        notes: Vec::new(),
                    },
                ),
                full_moon::Error::TokenizerError(error) => emit_codespan_locked(
                    &files,
                    &CodespanDiagnostic {
                        severity: CodespanSeverity::Error,
                        code: Some("parse_error".to_owned()),
                        message: match error.error() {
                            full_moon::tokenizer::TokenizerErrorType::UnclosedComment => {
                                "unclosed comment".to_string()
                            }
                            full_moon::tokenizer::TokenizerErrorType::UnclosedString => {
                                "unclosed string".to_string()
                            }
                            full_moon::tokenizer::TokenizerErrorType::UnexpectedShebang => {
                                "unexpected shebang".to_string()
                            }
                            full_moon::tokenizer::TokenizerErrorType::UnexpectedToken(
                                character,
                            ) => {
                                format!("unexpected character {}", character)
                            }
                            full_moon::tokenizer::TokenizerErrorType::InvalidSymbol(symbol) => {
                                format!("invalid symbol {}", symbol)
                            }
                        },
                        labels: vec![CodespanLabel::primary(
                            source_id,
                            codespan::Span::new(
                                error.position().bytes() as u32,
                                error.position().bytes() as u32,
                            ),
                        )],
                        notes: Vec::new(),
                    },
                ),
                _ => error!("Error parsing {}: {}", filename.display(), error),
            }

            return;
        }
    };

    let mut diagnostics = checker.test_on(&ast);
    diagnostics.sort_by_key(|diagnostic| diagnostic.diagnostic.start_position());

    let (mut errors, mut warnings) = (0, 0);
    for diagnostic in &diagnostics {
        match diagnostic.severity {
            Severity::Allow => {}
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
        };
    }

    LINT_ERRORS.fetch_add(errors, Ordering::SeqCst);
    LINT_WARNINGS.fetch_add(warnings, Ordering::SeqCst);

    let stdout = termcolor::StandardStream::stdout(get_color());
    let mut stdout = stdout.lock();

    for diagnostic in diagnostics {
        if opts.luacheck {
            // Existing Luacheck consumers presumably use --formatter plain
            let primary_label = &diagnostic.diagnostic.primary_label;
            let end = files.location(source_id, primary_label.range.1).unwrap();

            // Closures in Rust cannot call themselves recursively, especially not mutable ones.
            // Luacheck only allows one line ranges, so we just repeat the lint for every line it spans.
            // This would be frustrating for a human to read, but consumers (editors) will instead show it
            // as a native implementation would.
            let mut stack = Vec::new();

            let mut write = |stack: &mut Vec<_>, start: codespan::Location| -> io::Result<()> {
                write!(stdout, "{}:", filename.display())?;
                write!(stdout, "{}:{}", start.line.number(), start.column.number())?;

                if opts.ranges {
                    write!(
                        stdout,
                        "-{}",
                        if start.line != end.line {
                            // Report to the end of the line
                            files
                                .source(source_id)
                                .lines()
                                .nth(start.line.to_usize())
                                .unwrap()
                                .chars()
                                .count()
                        } else {
                            end.column.to_usize()
                        }
                    )?;
                }

                // The next line will be displayed just like this one
                if start.line != end.line {
                    stack.push(codespan::Location::new(
                        (start.line.to_usize() + 1) as u32,
                        0,
                    ));
                }

                write!(
                    stdout,
                    ": ({}000) ",
                    match diagnostic.severity {
                        Severity::Allow => return Ok(()),
                        Severity::Error => "E",
                        Severity::Warning => "W",
                    }
                )?;

                write!(stdout, "[{}] ", diagnostic.diagnostic.code)?;
                write!(stdout, "{}", diagnostic.diagnostic.message)?;

                if !diagnostic.diagnostic.notes.is_empty() {
                    write!(stdout, "\n{}", diagnostic.diagnostic.notes.join("\n"))?;
                }

                writeln!(stdout)?;
                Ok(())
            };

            write(
                &mut stack,
                files.location(source_id, primary_label.range.0).unwrap(),
            )
            .unwrap();

            while let Some(new_start) = stack.pop() {
                write(&mut stack, new_start).unwrap();
            }
        } else {
            let diagnostic = diagnostic.diagnostic.into_codespan_diagnostic(
                source_id,
                match diagnostic.severity {
                    Severity::Allow => continue,
                    Severity::Error => CodespanSeverity::Error,
                    Severity::Warning => CodespanSeverity::Warning,
                },
            );

            emit_codespan(&mut stdout, &files, &diagnostic);
        }
    }
}

fn read_file(checker: &Checker<toml::value::Value>, filename: &Path) {
    read(
        checker,
        filename,
        match fs::File::open(filename) {
            Ok(file) => file,
            Err(error) => {
                error!("Couldn't open file {}: {}", filename.display(), error);
                LINT_ERRORS.fetch_add(1, Ordering::SeqCst);
                return;
            }
        },
    );
}

fn start(mut matches: opts::Options) {
    *OPTIONS.write().unwrap() = Some(matches.clone());

    if matches.pattern.is_empty() {
        matches.pattern.push(String::from("**/*.lua"));
        #[cfg(feature = "roblox")]
        matches.pattern.push(String::from("**/*.luau"));
    }

    match matches.command {
        #[cfg(feature = "roblox")]
        Some(opts::Command::GenerateRobloxStd) => {
            println!("Generating Roblox standard library...");

            if let Err(error) = generate_roblox_std() {
                error!("Couldn't create Roblox standard library: {error:?}");
                std::process::exit(1);
            }

            return;
        }

        #[cfg(feature = "roblox")]
        Some(opts::Command::UpdateRobloxStd) => {
            println!("Updating Roblox standard library...");

            if let Err(error) = roblox::update_roblox_std() {
                error!("Couldn't update Roblox standard library: {error}");
                std::process::exit(1);
            }

            return;
        }

        Some(opts::Command::UpgradeStd { filename }) => {
            if let Err(error) = upgrade_std(filename) {
                error!("Couldn't upgrade standard library: {error}");
                std::process::exit(1);
            }

            return;
        }

        None => {}
    }

    let config: CheckerConfig<toml::value::Value> = match matches.config {
        Some(config_file) => {
            let config_contents = match fs::read_to_string(config_file) {
                Ok(contents) => contents,
                Err(error) => {
                    error!("Couldn't read config file: {}", error);
                    std::process::exit(1);
                }
            };

            match toml::from_str(&config_contents) {
                Ok(config) => config,
                Err(error) => {
                    error!("Config file not in correct format: {}", error);
                    std::process::exit(1);
                }
            }
        }

        None => match fs::read_to_string("selene.toml") {
            Ok(config_contents) => match toml::from_str(&config_contents) {
                Ok(config) => config,
                Err(error) => {
                    error!("Config file not in correct format: {}", error);
                    std::process::exit(1);
                }
            },

            Err(_) => CheckerConfig::default(),
        },
    };

    let current_dir = std::env::current_dir().unwrap();

    let standard_library =
        match standard_library::collect_standard_library(&config, &config.std, &current_dir) {
            Ok(Some(library)) => library,

            Ok(None) => {
                error!("Standard library was empty.");
                std::process::exit(1);
            }

            Err(error) => {
                let missing_files: Vec<_> = config
                    .std
                    .split('+')
                    .filter(|name| {
                        !PathBuf::from(format!("{name}.yml")).exists()
                            && !PathBuf::from(format!("{name}.toml")).exists()
                    })
                    .filter(|name| !cfg!(feature = "roblox") || *name != "roblox")
                    .collect();

                if !missing_files.is_empty() {
                    eprintln!(
                        "`std = \"{}\"`, but some libraries could not be found:",
                        config.std
                    );

                    for library_name in missing_files {
                        eprintln!("  `{library_name}`");
                    }

                    error!("Could not find all standard library files");
                    std::process::exit(1);
                }

                error!("Could not collect standard library: {error}");
                std::process::exit(1);
            }
        };

    let checker = Arc::new(match Checker::new(config, standard_library) {
        Ok(checker) => checker,
        Err(error) => {
            error!("{error}");
            std::process::exit(1);
        }
    });

    let pool = ThreadPool::new(matches.num_threads);

    let exclude_path = &matches.exclude_path.as_ref();

    for filename in &matches.files {
        if filename == "-" {
            let checker = Arc::clone(&checker);
            pool.execute(move || read(&checker, Path::new("-"), io::stdin().lock()));
            continue;
        }

        match fs::metadata(filename) {
            Ok(metadata) => {
                if metadata.is_file() {
                    let checker = Arc::clone(&checker);
                    let filename = filename.to_owned();

                    pool.execute(move || read_file(&checker, Path::new(&filename)));
                } else if metadata.is_dir() {
                    for pattern in &matches.pattern {
                        let glob = match glob::glob(&format!(
                            "{}/{}",
                            filename.to_string_lossy(),
                            pattern
                        )) {
                            Ok(glob) => glob,
                            Err(error) => {
                                error!("Invalid glob pattern: {}", error);
                                return;
                            }
                        };

                        for entry in glob {
                            match entry {
                                Ok(path) => {
                                    match exclude_path {
                                        Some(exclude) => {
                                            if path.starts_with(exclude) {
                                                continue;
                                            }
                                        }
                                        None => {}
                                    }

                                    let checker = Arc::clone(&checker);

                                    pool.execute(move || read_file(&checker, &path));
                                }

                                Err(error) => {
                                    error!(
                                        "Couldn't open file {}: {}",
                                        filename.to_string_lossy(),
                                        error
                                    );
                                }
                            };
                        }
                    }
                } else {
                    unreachable!("Somehow got a symlink from the files?");
                }
            }

            Err(error) => {
                error!(
                    "Error getting metadata of {}: {}",
                    filename.to_string_lossy(),
                    error
                );

                LINT_ERRORS.fetch_add(1, Ordering::SeqCst);
            }
        };
    }

    pool.join();

    let (parse_errors, lint_errors, lint_warnings) = (
        PARSE_ERRORS.load(Ordering::SeqCst),
        LINT_ERRORS.load(Ordering::SeqCst),
        LINT_WARNINGS.load(Ordering::SeqCst),
    );

    if !matches.luacheck && !matches.no_summary {
        log_total(parse_errors, lint_errors, lint_warnings).ok();
    }

    if parse_errors + lint_errors + lint_warnings + pool.panic_count() > 0 {
        std::process::exit(1);
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    #[cfg(feature = "tracy-profiling")]
    {
        tracy_client::Client::start();
    }

    let mut luacheck = false;

    if let Ok(path) = std::env::current_exe() {
        if let Some(stem) = path.file_stem() {
            if stem.to_str() == Some("luacheck") {
                luacheck = true;
            }
        }
    }

    start(get_opts(luacheck));

    Ok(())
}

// Will attempt to get the options.
// Different from Options::from_args() as if in Luacheck mode
// (either found from --luacheck or from the LUACHECK AtomicBool)
// it will ignore all extra parameters.
// If not in luacheck mode and errors are found, will exit.
fn get_opts(luacheck: bool) -> opts::Options {
    get_opts_safe(std::env::args_os().collect::<Vec<_>>(), luacheck)
        .unwrap_or_else(|err| err.exit())
}

fn get_opts_safe(mut args: Vec<OsString>, luacheck: bool) -> Result<opts::Options, clap::Error> {
    let mut first_error: Option<clap::Error> = None;

    loop {
        match opts::Options::from_iter_safe(&args) {
            Ok(mut options) => match first_error {
                Some(error) => {
                    if options.luacheck || luacheck {
                        options.luacheck = true;
                        break Ok(options);
                    } else {
                        break Err(error);
                    }
                }

                None => break Ok(options),
            },

            Err(err) => match err.kind {
                clap::ErrorKind::UnknownArgument => {
                    let bad_arg =
                        &err.info.as_ref().expect("no info for UnknownArgument")[0].to_owned();

                    args = args
                        .drain(..)
                        .filter(|arg| arg.to_string_lossy().split('=').next().unwrap() != bad_arg)
                        .collect();

                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }

                _ => break Err(err),
            },
        }
    }
}

#[cfg(feature = "roblox")]
fn generate_roblox_std() -> color_eyre::Result<StandardLibrary> {
    let (contents, std) = roblox::RobloxGenerator::generate()?;

    fs::File::create("roblox.yml").and_then(|mut file| file.write_all(&contents))?;

    Ok(std)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(mut args: Vec<&str>) -> Vec<OsString> {
        args.insert(0, "selene");
        args.into_iter().map(OsString::from).collect()
    }

    #[test]
    fn test_luacheck_opts() {
        assert!(get_opts_safe(args(vec!["file"]), false).is_ok());
        assert!(get_opts_safe(args(vec!["--fail", "files"]), false).is_err());

        match get_opts_safe(args(vec!["--luacheck", "--fail", "files"]), false) {
            Ok(opts) => {
                assert!(opts.luacheck);
                assert_eq!(opts.files, vec![OsString::from("files")]);
            }

            Err(err) => {
                panic!("selene --luacheck --fail files returned Err: {:?}", err);
            }
        }

        assert!(get_opts_safe(args(vec!["-", "--formatter=plain"]), true).is_ok());

        assert!(get_opts_safe(args(vec!["--fail", "files"]), true).is_ok());
    }
}
