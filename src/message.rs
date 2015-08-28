//! Generic message handling.
//!
//! This module provides the ability to read SBD messages from and write SBD messages to streams.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use Result;

/// An SBD message.
pub struct Message;

impl Message {

    /// Reads in a message from a file path.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::message::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Message> {
        let mut file = try!(File::open(path));
        Ok(Message)
    }

    /// Reads in a message from an object that implements `Read`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use sbd::message::Message;
    /// let mut file = File::open("data/0-mo.sbd").unwrap();
    /// let message = Message::read_from(file).unwrap();
    pub fn read_from<R: Read>(readable: R) -> Result<Message> {
        Ok(Message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;

    #[test]
    fn from_path() {
        Message::from_path("data/0-mo.sbd").unwrap();
    }

    #[test]
    fn from_read() {
        let mut file = File::open("data/0-mo.sbd").unwrap();
        Message::read_from(file).unwrap();
    }

    #[test]
    fn from_path_that_doesnt_exist() {
        assert!(Message::from_path("notafile.sbd").is_err());
    }
}
