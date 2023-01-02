use crate::opts::DisplayStyle;

use selene_lib::logs::{LogMessage, Logger};

struct Json2Logger;
struct RichLogger;

impl Logger for Json2Logger {
    fn log(&self, message: LogMessage) {
        crate::json_output::print_json(crate::json_output::JsonOutput::LogMessage(message))
    }
}

impl Logger for RichLogger {
    fn log(&self, message: LogMessage) {
        match message {
            LogMessage::WaitingForPluginSemaphoreDownload { semaphore_file } => {
                eprintln!(
                    "another instance of selene is currently downloading plugins.\nif this is stuck, delete `{}`",
                    semaphore_file.display()
                );
            }
        }
    }
}

pub fn get_logger(options: &crate::opts::Options) -> Option<Box<dyn Logger>> {
    match options.display_style() {
        DisplayStyle::Json2 => Some(Box::new(Json2Logger)),
        DisplayStyle::Rich => Some(Box::new(RichLogger)),

        // "Json" is used by old extensions, so we can't change its output
        DisplayStyle::Json => None,
        DisplayStyle::Quiet => None,
    }
}
