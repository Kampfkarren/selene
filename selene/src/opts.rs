use std::ffi::OsString;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[structopt(setting(structopt::clap::AppSettings::AllowExternalSubcommands))]
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

    /// Display only the necessary information
    #[structopt(long, short)]
    pub quiet: bool,

    /// Whether to pretend to be luacheck for existing consumers
    #[structopt(long, hidden(true))]
    pub luacheck: bool,

    #[structopt(parse(from_os_str), min_values(1), index(1), required(true))]
    pub files: Vec<OsString>,
}

// We can't just do default_value = num_cpus::get().to_string().as_str(),
// since that won't extend the lifetime for long enough.
fn get_num_cpus() -> &'static str {
    Box::leak(num_cpus::get().to_string().into_boxed_str())
}
