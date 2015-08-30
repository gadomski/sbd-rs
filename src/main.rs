//! Command line utility for querying and working with Iridium SBD messages.

extern crate docopt;
extern crate rustc_serialize;

use std::path::PathBuf;

use docopt::Docopt;

const USAGE: &'static str = "
Iridium Short Burst Data (SBD) message utility.

Usage:
    sbd list <directory>
    sbd (-h | --help)
    sbd --version

Options:
    -h --help   Show this information
    --version   Show version
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_directory: PathBuf,
}

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| Ok(d.version(Some(env!("CARGO_PKG_VERSION").to_string()))))
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
}
