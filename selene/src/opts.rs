use std::ffi::OsString;

use structopt::{clap::arg_enum, StructOpt};

#[derive(Clone, Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[structopt(setting(structopt::clap::AppSettings::ArgsNegateSubcommands))]
#[structopt(setting(structopt::clap::AppSettings::SubcommandsNegateReqs))]
pub struct Options {
    /// A glob to match files with to check
    #[structopt(long, default_value = "**/*.lua")]
    pub pattern: String,

    /// A toml file to configure the behavior of selene [default: selene.toml]
    // .default is not used here since if the user explicitly specifies the config file
    // we want it to error if it doesn't exist
    #[structopt(long)]
    pub config: Option<String>,

    /// Number of threads to run on, default to the numbers of logical cores on your system
    #[structopt(long, default_value = get_num_cpus())]
    pub num_threads: usize,

    /// Sets the display method
    #[structopt(
        long,
        possible_values = &DisplayStyle::variants(),
        case_insensitive = true,
        conflicts_with = "quiet",
        default_value = "rich",
    )]
    pub display_style: DisplayStyle,

    /// Display only the necessary information.
    /// Equivalent to --display-style="quiet"
    #[structopt(long, short)]
    pub quiet: bool,

    #[structopt(
        long,
        possible_values = &Color::variants(),
        case_insensitive = true,
        default_value = "auto",
    )]
    pub color: Color,

    /// Whether to pretend to be luacheck for existing consumers
    #[structopt(long, hidden(true))]
    pub luacheck: bool,

    // Only used in Luacheck mode
    #[structopt(long, hidden(true))]
    pub ranges: bool,

    #[structopt(parse(from_os_str), min_values(1), index(1), required(true))]
    pub files: Vec<OsString>,

    #[structopt(subcommand)]
    pub command: Option<Command>,
}

impl Options {
    pub fn quiet(&self) -> bool {
        self.quiet || self.display_style == DisplayStyle::Quiet
    }
}

#[derive(Clone, Debug, PartialEq, Eq, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Command {
    #[cfg(feature = "roblox")]
    GenerateRobloxStd {
        #[structopt(long)]
        deprecated: bool,
    },
}

arg_enum! {
    #[derive(Clone, Copy, Debug)]
    pub enum Color {
        Always,
        Auto,
        Never,
    }
}

arg_enum! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum DisplayStyle {
        Json,
        Rich,
        Quiet,
    }
}

// We can't just do default_value = num_cpus::get().to_string().as_str(),
// since that won't extend the lifetime for long enough.
fn get_num_cpus() -> &'static str {
    Box::leak(num_cpus::get().to_string().into_boxed_str())
}
