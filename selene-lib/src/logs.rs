use std::path::PathBuf;

use once_cell::sync::OnceCell;
use serde::Serialize;

static LOGGER: OnceCell<Box<dyn Logger>> = OnceCell::new();

pub trait Logger: Send + Sync {
    fn log(&self, message: LogMessage);
}

#[derive(Serialize)]
pub enum LogMessage {
    WaitingForPluginSemaphoreDownload { semaphore_file: PathBuf },
}

pub fn set_logger(logger: Box<dyn Logger>) {
    if LOGGER.set(logger).is_err() {
        unreachable!("logger already set");
    }
}

pub(crate) fn log(message: LogMessage) {
    if let Some(logger) = LOGGER.get() {
        logger.log(message);
    }
}
