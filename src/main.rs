//! Command line utility for querying and working with Iridium SBD messages.

extern crate chrono;
extern crate docopt;
extern crate log;
extern crate rustc_serialize;
extern crate sbd;

use std::io::Write;
use std::path::Path;
use std::process;
use std::str;

use docopt::Docopt;

use sbd::directip::Server;
use sbd::storage::FilesystemStorage;
use sbd::mo::Message;

const USAGE: &'static str = "
Iridium Short Burst Data (SBD) message utility.

Usage:
    sbd \
                             read <file>
    sbd serve <addr> <directory> [--logfile=<logfile>]
    \
                             sbd (-h | --help)
    sbd --version

Options:
    -h --help               \
                             Show this information
    --version               Show version
    \
                             --logfile=<logfile>     Logfile [default: /var/log/iridiumd.log]
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_read: bool,
    cmd_serve: bool,
    arg_addr: String,
    arg_directory: String,
    arg_file: String,
    flag_logfile: String,
}

struct Logger<P: AsRef<Path>> {
    path: P,
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
                                   chrono::UTC::now().format("%Y-%m-%d %H:%M:%S"),
                                   record.level(),
                                   record.args())
                               .as_bytes())
                .unwrap();
        }
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                         .and_then(|d| Ok(d.version(Some(env!("CARGO_PKG_VERSION").to_string()))))
                         .and_then(|d| d.decode())
                         .unwrap_or_else(|e| e.exit());

    if args.cmd_read {
        match Message::from_path(&args.arg_file) {
            Ok(message) => {
                println!("{}", str::from_utf8(message.payload_ref()).unwrap());
            }
            Err(err) => println!("ERROR: {:?}", err),
        }
    }
    if args.cmd_serve {
        match log::set_logger(|max_log_level| {
            max_log_level.set(log::LogLevelFilter::Debug);
            Box::new(Logger { path: args.flag_logfile.clone() })
        }) {
            Ok(()) => {}
            Err(err) => {
                println!("Error when creating logger: {:?}", err);
                process::exit(1);
            }
        };
        let storage = FilesystemStorage::open(&args.arg_directory).unwrap_or_else(|e| {
            println!("Error when opening fileystem storage: {}", e);
            process::exit(1);
        });
        let mut server = Server::new(&args.arg_addr[..], storage);
        match server.bind() {
            Ok(()) => server.serve_forever(),
            Err(err) => {
                println!("Error when trying to bind to socket: {:?}", err);
                process::exit(1);
            }
        }
    }
}
