//! A filesystem logger for use (primarily) with the DirectIP server.

use std::path::Path;

use log::{Log, LogLevel, LogLevelFilter, LogMetadata, LogRecord, set_logger, SetLoggerError};

pub fn init<P: 'static + AsRef<Path> + Send + Sync>(path: P) -> Result<(), SetLoggerError> {
    set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Debug);
        Box::new(Logger { path: path })
    })
}

struct Logger<P: AsRef<Path>> {
    path: P,
}

impl<P: AsRef<Path> + Send + Sync> Log for Logger<P> {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Debug
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}
