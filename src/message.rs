//! Generic message handling.
//!
//! This module provides the ability to read SBD messages from and write SBD messages to streams.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use byteorder::{ReadBytesExt, BigEndian};

use {Error, Result};
use information_element::InformationElement;

/// An SBD message.
#[derive(Debug, Default)]
pub struct Message {
    protocol_revision_number: u8,
    overall_message_length: u16,
    information_elements: Vec<InformationElement>,
}

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
        let file = try!(File::open(path));
        Message::read_from(file)
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
    pub fn read_from<R: Read>(mut readable: R) -> Result<Message> {
        let mut message: Message = Default::default();
        message.protocol_revision_number = try!(readable.read_u8());
        if message.protocol_revision_number != 1 {
            return Err(Error::InvalidProtocolRevisionNumber(message.protocol_revision_number));
        }
        message.overall_message_length = try!(readable.read_u16::<BigEndian>());
        Ok(message)
    }

    /// Returns the overall message length of the SBD message.
    ///
    /// This value *includes* the initial three bytes, whereas the `overall_message_length`
    /// value in the SBD header does *not* include those bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::message::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// assert_eq!(59, message.len());
    /// ```
    pub fn len(&self) -> usize {
        self.overall_message_length as usize + 3
    }

    /// Returns true if this message is mobile originated.
    ///
    /// This is deteremined by the set of information elements in this message.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::message::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// assert!(message.is_mobile_originated());
    /// ```
    pub fn is_mobile_originated(&self) -> bool {
        true
    }

    /// Returns true if this message is mobile terminated.
    ///
    /// This is deteremined by the set of information elements in this message.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::message::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// assert!(!message.is_mobile_terminated());
    /// ```
    pub fn is_mobile_terminated(&self) -> bool {
        false
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
        let file = File::open("data/0-mo.sbd").unwrap();
        Message::read_from(file).unwrap();
    }

    #[test]
    fn from_path_that_doesnt_exist() {
        assert!(Message::from_path("notafile.sbd").is_err());
    }

    #[test]
    fn from_path_that_is_not_an_sbd_message() {
        assert!(Message::from_path("data/1-invalid.sbd").is_err());
    }

    #[test]
    fn len() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        assert_eq!(59, message.len());
    }

    #[test]
    fn is_mobile_originated() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        assert!(message.is_mobile_originated());
        assert!(!message.is_mobile_terminated());
        // TODO try to get a mobile terminated message to test the other way
    }
}
