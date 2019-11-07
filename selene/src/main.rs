use std::{
    ffi::OsString,
    fmt, fs,
    io::{self, Read, Write},
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

use codespan_reporting::{diagnostic::Severity as CodespanSeverity, term::DisplayStyle};
use full_moon::ast::owned::Owned;
use selene_lib::{rules::Severity, standard_library::StandardLibrary, *};
use structopt::{clap, StructOpt};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use threadpool::ThreadPool;

mod opts;

macro_rules! error {
    ($fmt:expr) => {
        error(fmt::format(format_args!($fmt))).unwrap();
    };

    ($fmt:expr, $($args:tt)*) => {
        error(fmt::format(format_args!($fmt, $($args)*))).unwrap();
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

fn error(text: String) -> io::Result<()> {
    let mut stderr = StandardStream::stderr(get_color());
    stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    write!(&mut stderr, "ERROR: ")?;
    stderr.reset()?;
    writeln!(&mut stderr, "{}", text)?;
    Ok(())
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

fn read<R: Read>(checker: &Checker<toml::value::Value>, filename: &Path, mut reader: R) {
    let mut buffer = Vec::new();
    if let Err(error) = reader.read_to_end(&mut buffer) {
        error!(
            "Couldn't read contents of file {}: {}",
            filename.display(),
            error,
        );
    }

    let contents = String::from_utf8_lossy(&buffer);

    let lock = OPTIONS.read().unwrap();
    let opts = lock.as_ref().unwrap();

    let ast = match full_moon::parse(&contents) {
        Ok(ast) => ast.owned(),
        Err(error) => {
            // TODO: Use codespan for this
            PARSE_ERRORS.fetch_add(1, Ordering::Release);
            error!("Error parsing {}: {}", filename.display(), error);
            return;
        }
    };

    let mut diagnostics = checker.test_on(&ast);
    diagnostics.sort_by_key(|diagnostic| diagnostic.diagnostic.start_position());

    let mut files = codespan::Files::new();
    let source_id = files.add(filename.to_string_lossy(), contents);

    let stdout = termcolor::StandardStream::stdout(get_color());
    let mut stdout = stdout.lock();

    let (mut errors, mut warnings) = (0, 0);
    for diagnostic in &diagnostics {
        match diagnostic.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
        };
    }

    LINT_ERRORS.fetch_add(errors, Ordering::Release);
    LINT_WARNINGS.fetch_add(warnings, Ordering::Release);

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
                    Severity::Error => CodespanSeverity::Error,
                    Severity::Warning => CodespanSeverity::Warning,
                },
            );

            codespan_reporting::term::emit(
                &mut stdout,
                &codespan_reporting::term::Config {
                    display_style: if opts.quiet {
                        DisplayStyle::Short
                    } else {
                        DisplayStyle::Rich
                    },
                    ..Default::default()
                },
                &files,
                &diagnostic,
            )
            .expect("couldn't emit to codespan");
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
                error!("Couldn't open file {}: {}", filename.display(), error,);

                return;
            }
        },
    );
}

fn start(matches: opts::Options) {
    *OPTIONS.write().unwrap() = Some(matches.clone());

    let config: CheckerConfig<toml::value::Value> = match matches.config {
        Some(config_file) => {
            let config_contents = match fs::read_to_string(config_file) {
                Ok(contents) => contents,
                Err(error) => {
                    error!("Couldn't read config file: {}", error);
                    return;
                }
            };

            match toml::from_str(&config_contents) {
                Ok(config) => config,
                Err(error) => {
                    error!("Config file not in correct format: {}", error);
                    return;
                }
            }
        }

        None => match fs::read_to_string("selene.toml") {
            Ok(config_contents) => match toml::from_str(&config_contents) {
                Ok(config) => config,
                Err(error) => {
                    error!("Config file not in correct format: {}", error);
                    return;
                }
            },

            Err(_) => CheckerConfig::default(),
        },
    };

    let current_dir = std::env::current_dir().unwrap();
    let standard_library = match StandardLibrary::from_config_name(&config.std, Some(&current_dir))
    {
        Ok(Some(library)) => library,

        Ok(None) => {
            error!("Standard library was empty.");
            return;
        }

        Err(error) => {
            error!("Could not retrieve standard library: {}", error);
            return;
        }
    };

    let checker = Arc::new(match Checker::new(config, standard_library) {
        Ok(checker) => checker,
        Err(error) => {
            error!("{}", error);
            return;
        }
    });

    let pool = ThreadPool::new(matches.num_threads);

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
                    let glob = match glob::glob(&format!(
                        "{}/{}",
                        filename.to_string_lossy(),
                        matches.pattern,
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
            }
        };
    }

    pool.join();

    let (parse_errors, lint_errors, lint_warnings) = (
        PARSE_ERRORS.load(Ordering::Relaxed),
        LINT_ERRORS.load(Ordering::Relaxed),
        LINT_WARNINGS.load(Ordering::Relaxed),
    );

    if !matches.luacheck {
        log_total(parse_errors, lint_errors, lint_warnings).ok();
    }

    if parse_errors + lint_errors + lint_warnings > 0 {
        std::process::exit(1);
    }
}

fn main() {
    let mut luacheck = false;

    if let Ok(path) = std::env::current_exe() {
        if let Some(stem) = path.file_stem() {
            if stem.to_str() == Some("luacheck") {
                luacheck = true;
            }
        }
    }

    start(get_opts(luacheck));
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
