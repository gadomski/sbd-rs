//! A filesystem logger for use (primarily) with the DirectIP server.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use log::{Log, LogLevel, LogLevelFilter, LogMetadata, LogRecord, set_logger, SetLoggerError};
use time;

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

    /// Log a message.
    ///
    /// This function has some panics in it. I'm not sure of the "right" way to handle exceptional
    /// situaions in this logging module. Part of me wants to ignore everything, since logging
    /// should not interfere with the functioning of the program as a whole. However, since I'm in
    /// dev mode for the whole system, silent logs might be worse than a crashing program. For now,
    /// I'll keep the panics, but with the idea that I need to fix this in the future.
    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&self.path).unwrap();
            file.write_all(format!("({}) {}: {}\n", time::now_utc().strftime("%Y-%m-%d %H:%M:%S").unwrap(),
                record.level(), record.args()).as_bytes()).unwrap();
        }
    }
}
