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

fn error(text: String) -> io::Result<()> {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    write!(&mut stderr, "ERROR: ")?;
    stderr.set_color(ColorSpec::new().set_fg(None))?;
    writeln!(&mut stderr, "{}", text)?;
    Ok(())
}

fn log_total(parse_errors: usize, lint_errors: usize, lint_warnings: usize) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    stdout.set_color(ColorSpec::new().set_fg(None))?;
    writeln!(&mut stdout, "Results:")?;

    let mut stat = |number: usize, label: &str| -> io::Result<()> {
        if number > 0 {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        } else {
            stdout.set_color(ColorSpec::new().set_fg(None))?;
        }

        write!(&mut stdout, "{}", number)?;
        stdout.set_color(ColorSpec::new().set_fg(None))?;
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

    let stdout = termcolor::StandardStream::stdout(termcolor::ColorChoice::Auto);
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
            write!(stdout, "{}:", filename.display()).unwrap();

            let primary_label = &diagnostic.diagnostic.primary_label;
            let location = files.location(source_id, primary_label.range.0).unwrap();
            write!(
                stdout,
                "{}:{}",
                location.line.to_usize() + 1,
                location.column.to_usize() + 1
            )
            .unwrap();

            if opts.ranges {
                let end_location = files.location(source_id, primary_label.range.1).unwrap();

                write!(
                    stdout,
                    "-{}",
                    if location.line != end_location.line {
                        // Cool that Luacheck only allows one line ranges :upside_down:
                        // Keep going byte after byte until we get to a new line,
                        // so that we capture the whole line
                        let mut offset = 0;

                        loop {
                            let check =
                                match files.location(source_id, primary_label.range.1 + offset) {
                                    Ok(location) => location,
                                    Err(_) => {
                                        break location.column.to_usize() + offset as usize;
                                    }
                                };

                            if check.line != location.line {
                                // The offset before this was the last before going to a new line
                                break check.column.to_usize() + offset as usize + 1;
                            }

                            offset += 1;
                        }
                    } else {
                        end_location.column.to_usize()
                    }
                )
                .unwrap();
            }

            write!(
                stdout,
                ": ({}000) ",
                match diagnostic.severity {
                    Severity::Error => "E",
                    Severity::Warning => "W",
                }
            )
            .unwrap();
            write!(stdout, "[{}] ", diagnostic.diagnostic.code).unwrap();
            write!(stdout, "{}", diagnostic.diagnostic.message).unwrap();

            if !diagnostic.diagnostic.notes.is_empty() {
                write!(stdout, "\n{}", diagnostic.diagnostic.notes.join("\n")).unwrap();
            }

            writeln!(stdout).unwrap();
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

    let standard_library = match fs::read_to_string(format!("{}.toml", &config.std)) {
        Ok(contents) => match toml::from_str::<StandardLibrary>(&contents) {
            Ok(mut standard_library) => {
                standard_library.inflate();
                standard_library
            }
            Err(error) => {
                error!(
                    "Custom standard library wasn't formatted properly: {}",
                    error
                );
                return;
            }
        },

        Err(_) => match StandardLibrary::from_name(&config.std) {
            Some(std) => std,
            None => {
                error!("Unknown standard library '{}'", config.std);
                return;
            }
        },
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
