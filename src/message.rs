//! Generic message handling.
//!
//! This module provides the ability to read SBD messages from and write SBD messages to streams.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

use {Error, Result};
use information_element::{InformationElement, InformationElementType};

const MESSAGE_HEADER_LENGTH: usize = 3;

/// An SBD message.
#[derive(Debug, Default)]
pub struct Message {
    protocol_revision_number: u8,
    overall_message_length: u16,
    information_elements: HashMap<InformationElementType, InformationElement>,
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
    /// Per the specification, oversized and undersized messages will result in an error.
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
        let mut bytes_read = 0u16;
        loop {
            let ie = match InformationElement::read_from(&mut readable) {
                Ok(ie) => ie,
                Err(e) => return Err(e),
            };
            bytes_read += ie.len();
            message.information_elements.insert(ie.id(), ie);
            if bytes_read >= message.overall_message_length {
                break
            }
        }
        if try!(readable.take(1).read_to_end(&mut Vec::new())) != 0 {
            return Err(Error::Oversized);
        }
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
        self.overall_message_length as usize + MESSAGE_HEADER_LENGTH
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
        // TODO this is a placeholder
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
        // TODO this is a placeholder
        false
    }

    /// Convert this message into its information elements.
    ///
    /// This allows extraction of components of the information elements easily.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::message::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let information_elements = message.into_information_elements();
    /// assert_eq!(2, information_elements.len());
    /// ```
    pub fn into_information_elements(self) -> HashMap<InformationElementType, InformationElement> {
        self.information_elements
    }

    /// Write this message back to a object that can `Write`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use sbd::message::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let mut cursor = Cursor::new(Vec::new());
    /// message.write_to(&mut cursor);
    /// ```
    pub fn write_to<W: Write>(&self, w: &mut W) -> Result<()> {
        try!(w.write_u8(self.protocol_revision_number));
        try!(w.write_u16::<BigEndian>(self.overall_message_length));
        for information_element in self.information_elements.values() {
            try!(w.write_u8(information_element.id() as u8));
            let contents = information_element.contents_ref();
            try!(w.write_u16::<BigEndian>(contents.len() as u16));
            try!(w.write_all(&contents[..]));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::{Cursor, Read};

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

    #[test]
    fn undersized() {
        let file = File::open("data/0-mo.sbd").unwrap();
        let readable = file.take(58);
        assert!(Message::read_from(readable).is_err());
    }

    #[test]
    fn oversized() {
        let file = File::open("data/0-mo.sbd").unwrap();
        let readable = file.chain(Cursor::new(vec![0]));
        assert!(Message::read_from(readable).is_err());
    }

    #[test]
    fn write() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        let mut cursor = Cursor::new(Vec::new());
        message.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        Message::read_from(cursor).unwrap();
    }
}
