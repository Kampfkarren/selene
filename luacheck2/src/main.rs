use std::{
    env::current_dir,
    fmt, fs,
    io::{self, Write},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use clap::{App, Arg};
use codespan_reporting::{diagnostic::Severity as CodespanSeverity, term::DisplayStyle};
use full_moon::ast::owned::Owned;
use luacheck2_lib::{rules::Severity, standard_library::StandardLibrary, *};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use threadpool::ThreadPool;

macro_rules! error {
    ($fmt:expr) => {
        error(fmt::format(format_args!($fmt))).unwrap();
    };

    ($fmt:expr, $($args:tt)*) => {
        error(fmt::format(format_args!($fmt, $($args)*))).unwrap();
    };
}

static QUIET: AtomicBool = AtomicBool::new(false);

fn error(text: String) -> io::Result<()> {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    write!(&mut stderr, "ERROR: ")?;
    stderr.set_color(ColorSpec::new().set_fg(None))?;
    writeln!(&mut stderr, "{}", text)?;
    Ok(())
}

fn read_file(checker: &Checker<toml::value::Value>, filename: &Path) {
    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(error) => {
            error!(
                "Couldn't read contents of file {}: {}",
                filename.display(),
                error
            );
            return;
        }
    };

    let ast = match full_moon::parse(&contents) {
        Ok(ast) => ast.owned(),
        Err(error) => {
            // TODO: Use codespan for this
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

    for diagnostic in diagnostics.into_iter().map(|diagnostic| {
        diagnostic.diagnostic.into_codespan_diagnostic(
            source_id,
            match diagnostic.severity {
                Severity::Error => CodespanSeverity::Error,
                Severity::Warning => CodespanSeverity::Warning,
            },
        )
    }) {
        codespan_reporting::term::emit(
            &mut stdout,
            &codespan_reporting::term::Config {
                display_style: if QUIET.load(Ordering::Relaxed) {
                    DisplayStyle::Rich
                } else {
                    DisplayStyle::Short
                },
                ..Default::default()
            },
            &files,
            &diagnostic,
        )
        .expect("couldn't emit to codespan");
    }
}

fn main() {
    let num_cpus = num_cpus::get().to_string();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Kampfkarren")
        .arg(
            Arg::with_name("pattern")
                .long("pattern")
                .help("A glob to match files with to check")
                .default_value("**/*.lua"),
        )
        .arg(
            // .default is not used here since if the user explicitly specifies the config file
            // we want it to error if it doesn't exist
            Arg::with_name("config")
                .long("config")
                .help(
                    "A toml file to configure the behavior of luacheck2 [default: luacheck2.toml]",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("num-threads")
                .long("num-threads")
                .help(
                    "Number of threads to run on, default to the numbers of logical cores on your system"
                )
                .default_value(&num_cpus)
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .help("Display only the necessary information")
        )
        .arg(
            Arg::with_name("files")
                .index(1)
                .min_values(1)
                .multiple(true)
                .required(true),
        )
        .get_matches();

    QUIET.store(!matches.is_present("quiet"), Ordering::Relaxed);

    let config: CheckerConfig<toml::value::Value> = match matches.value_of("config") {
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

        None => match fs::read_to_string("luacheck2.toml") {
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
        Ok(contents) => match toml::from_str(&contents) {
            Ok(standard_library) => standard_library,
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

    let num_threads: usize = match &matches.value_of("num-threads").unwrap().parse() {
        Ok(num_threads) => *num_threads,
        Err(error) => {
            error!("Couldn't parse num-threads: {}", error);
            return;
        }
    };

    let pattern = matches.value_of("pattern").unwrap();

    let pool = ThreadPool::new(num_threads);

    for filename in matches.values_of_os("files").unwrap() {
        match fs::metadata(filename) {
            Ok(metadata) => {
                if metadata.is_file() {
                    let checker = Arc::clone(&checker);

                    pool.execute(move || {
                        read_file(
                            &checker,
                            &current_dir().expect("Failed to get current directory"),
                        )
                    });
                } else if metadata.is_dir() {
                    let glob =
                        match glob::glob(&format!("{}/{}", filename.to_string_lossy(), pattern)) {
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
}
