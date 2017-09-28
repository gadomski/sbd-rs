//! Command line utility for querying and working with Iridium SBD messages.

extern crate chrono;
extern crate docopt;
extern crate log;
extern crate sbd;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use docopt::Docopt;
use sbd::directip::Server;
use sbd::mo::{Message, SessionStatus};
use sbd::storage::FilesystemStorage;
use std::io::Write;
use std::path::Path;
use std::process;
use std::str;

const USAGE: &'static str = "
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
    fn enabled(&self, metadata: &log::LogMetadata) -> bool {
        metadata.level() <= log::LogLevel::Debug
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.metadata()) {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&self.path)
                .unwrap();
            file.write_all(format!("({}) {}: {}\n",
                                   chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                                   record.level(),
                                   record.args())
                                   .as_bytes())
                .unwrap();
        }
    }
}

impl ReadableMessage {
    fn new(m: &Message) -> ReadableMessage {
        ReadableMessage {
            protocol_revision_number: m.protocol_revision_number(),
            cdr_reference: m.cdr_reference(),
            imei: m.imei().to_string(),
            session_status: m.session_status(),
            momsn: m.momsn(),
            mtmsn: m.mtmsn(),
            time_of_session: m.time_of_session().to_rfc2822(),
            payload: str::from_utf8(m.payload_ref()).unwrap_or("<binary payload>").to_string(),
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| Ok(d.version(Some(env!("CARGO_PKG_VERSION").to_string()))))
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
        match Message::from_path(&args.arg_file) {
            Ok(message) => {
                print!("{}",
                         str::from_utf8(message.payload_ref()).unwrap_or_else(|e| {
                             println!("ERROR: Unable to convert payload to utf8: {}", e);
                             process::exit(1);
                         }));
            }
            Err(err) => {
                println!("ERROR: Unable to extract payload: {}", err);
                process::exit(1);
            }
        }
    }
    if args.cmd_serve {
        log::set_logger(|max_log_level| {
                            max_log_level.set(log::LogLevelFilter::Debug);
                            Box::new(Logger { path: args.flag_logfile.clone() })
                        })
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
