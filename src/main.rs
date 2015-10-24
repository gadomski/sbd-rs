//! Command line utility for querying and working with Iridium SBD messages.

extern crate docopt;
extern crate rustc_serialize;
extern crate sbd;

use std::str;

use docopt::Docopt;

use sbd::filesystem::Storage;
use sbd::message::Message;

const USAGE: &'static str = "
Iridium Short Burst Data (SBD) message utility.

Usage:
    sbd list <directory>
    sbd read <file>
    sbd (-h | --help)
    sbd --version

Options:
    -h --help   Show this information
    --version   Show version
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_list: bool,
    cmd_read: bool,
    arg_directory: String,
    arg_file: String,
}

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| Ok(d.version(Some(env!("CARGO_PKG_VERSION").to_string()))))
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    if args.cmd_list {
        for entry in &Storage::new(args.arg_directory) {
            println!("{}", entry.path_buf.to_str().unwrap());
        }
    }
    if args.cmd_read {
        match Message::from_path(args.arg_file) {
            Ok(message) => {
                println!("{}", str::from_utf8(message.payload_ref()).unwrap());
            }
            Err(err) => println!("ERROR: {:?}", err),
        }
    }
}
