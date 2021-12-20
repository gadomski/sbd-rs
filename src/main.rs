//! Command line utility for querying and working with Iridium SBD messages.

extern crate chrono;
extern crate docopt;
extern crate log;
extern crate sbd;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::{io::Write, path::Path, process, str};

use docopt::Docopt;
use sbd::{
    directip::Server,
    mo::{Message, SessionStatus},
    storage::FilesystemStorage,
};

const USAGE: &str = "
Iridium Short Burst Data (SBD) message utility.

Usage:
    sbd info <file> [--compact]
    sbd payload <file>
    sbd serve <addr> <directory> [--logfile=<logfile>]
    sbd (-h | --help)
    sbd --version

Options:
    -h --help               Show this information
    --version               Show version
    --logfile=<logfile>     Logfile [default: /var/log/iridiumd.log]
    --compact               Don't pretty-print the JSON
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_info: bool,
    cmd_payload: bool,
    cmd_serve: bool,
    arg_addr: String,
    arg_directory: String,
    arg_file: String,
    flag_logfile: String,
    flag_compact: bool,
}

struct Logger<P: AsRef<Path>> {
    path: P,
}

#[derive(Debug, Serialize)]
struct ReadableMessage {
    protocol_revision_number: u8,
    cdr_reference: u32,
    imei: String,
    session_status: SessionStatus,
    momsn: u16,
    mtmsn: u16,
    time_of_session: String,
    payload: String,
}

impl<P: AsRef<Path> + Send + Sync> log::Log for Logger<P> {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&self.path)
                .unwrap();
            file.write_all(
                format!(
                    "({}) {}: {}\n",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.args()
                )
                .as_bytes(),
            )
            .unwrap();
        }
    }

    fn flush(&self) {}
}

impl ReadableMessage {
    fn new(_: &Message) -> ReadableMessage {
        unimplemented!()
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .map(|d| d.version(Some(env!("CARGO_PKG_VERSION").to_string())))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.cmd_info {
        match Message::from_path(&args.arg_file) {
            Ok(ref message) => {
                let message = &ReadableMessage::new(message);
                if args.flag_compact {
                    println!("{}", serde_json::to_string(message).unwrap());
                } else {
                    println!("{}", serde_json::to_string_pretty(message).unwrap());
                };
            }
            Err(err) => {
                println!("ERROR: Unable to read message: {}", err);
                process::exit(1);
            }
        }
    }
    if args.cmd_payload {
        unimplemented!()
    }
    if args.cmd_serve {
        let logger = Logger {
            path: args.flag_logfile.clone(),
        };
        log::set_boxed_logger(Box::new(logger))
            .map(|()| log::set_max_level(log::LevelFilter::Debug))
            .unwrap_or_else(|e| {
                println!("ERROR: Could not create logger: {}", e);
                process::exit(1);
            });
        let storage = FilesystemStorage::open(args.arg_directory).unwrap_or_else(|e| {
            println!("ERROR: Could not open storage: {}", e);
            process::exit(1);
        });
        let mut server = Server::new(&args.arg_addr[..], storage);
        match server.bind() {
            Ok(()) => server.serve_forever(),
            Err(err) => {
                println!("ERROR: Could not bind to socket: {}", err);
                process::exit(1);
            }
        }
    }
}
