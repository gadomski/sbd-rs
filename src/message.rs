//! Module for reading and writing SBD messages.
//!
//! Though messages technically come in two flavors, mobile originated and mobile terminated, we
//! only handle mobile originated messages in this library.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::str;
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt};
use chrono::{DateTime, Duration, TimeZone, UTC};
use num::traits::FromPrimitive;

use super::{Error, Result};
use super::information_element::{InformationElement, InformationElementType};

/// The protocol number of an SBD message.
///
/// At this point, this can *only* be one.
#[derive(Debug, Default, PartialEq)]
struct ProtocolRevisionNumber(u8);

impl ProtocolRevisionNumber {
    /// Returns true if this is a valid protocol revision number.
    pub fn valid(&self) -> bool {
        self.0 == 1
    }
}

/// The modem IMEI identifier.
#[derive(Debug, PartialEq)]
struct Imei([u8; 15]);

impl Default for Imei {
    fn default() -> Imei {
        Imei([0; 15])
    }
}

impl Imei {
    /// Returns this IMEI number as a `str`.
    ///
    /// # Panics
    ///
    /// Panics if the string is not valid utf8.
    fn as_str(&self) -> &str {
        str::from_utf8(&self.0).unwrap()
    }
}

/// The status of a mobile-originated session.
enum_from_primitive! {
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SessionStatus {
    Ok = 0,
    OkMobileTerminatedTooLarge = 1,
    OkLocationUnacceptableQuality = 2,
    Timeout = 10,
    MobileOriginatedTooLarge = 12,
    RFLinkLoss = 13,
    IMEIProtocolAnomaly = 14,
    Prohibited = 15,
    Unknown,
}
}

impl Default for SessionStatus {
    fn default() -> SessionStatus {
        SessionStatus::Unknown
    }
}

/// A mobile-origined Iridium SBD message.
#[derive(Debug, PartialEq)]
pub struct Message {
    protocol_revision_number: ProtocolRevisionNumber,
    cdr_reference: u32,
    imei: Imei,
    session_status: SessionStatus,
    momsn: u16,
    mtmsn: u16,
    time_of_session: DateTime<UTC>,
    payload: Vec<u8>,
}

impl Default for Message {
    fn default() -> Message {
        Message {
            protocol_revision_number: Default::default(),
            cdr_reference: Default::default(),
            imei: Default::default(),
            session_status: Default::default(),
            momsn: Default::default(),
            mtmsn: Default::default(),
            time_of_session: UTC.ymd(1970, 1, 1).and_hms(0, 0, 0),
            payload: Vec::new(),
        }
    }
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

        message.protocol_revision_number = ProtocolRevisionNumber(try!(readable.read_u8()));
        if !message.protocol_revision_number.valid() {
            return Err(Error::InvalidProtocolRevisionNumber(message.protocol_revision_number.0));
        }
        let overall_message_length = try!(readable.read_u16::<BigEndian>());

        let mut information_elements: HashMap<InformationElementType, InformationElement> = HashMap::new();
        let mut bytes_read = 0u16;
        loop {
            let ie = match InformationElement::read_from(&mut readable) {
                Ok(ie) => ie,
                Err(e) => return Err(e),
            };
            bytes_read += ie.len();
            information_elements.insert(ie.id(), ie);
            if bytes_read >= overall_message_length {
                break
            }
        }

        if try!(readable.take(1).read_to_end(&mut Vec::new())) != 0 {
            return Err(Error::Oversized);
        }

        let header = match information_elements.remove(&InformationElementType::MobileOriginatedHeader) {
            Some(ie) => ie,
            None => return Err(Error::MissingMobileOriginatedHeader),
        };
        let mut cursor = &mut Cursor::new(header.contents_ref());
        message.cdr_reference = try!(cursor.read_u32::<BigEndian>());
        let bytes_read = try!(cursor.take(message.imei.0.len() as u64).read(&mut message.imei.0));
        if bytes_read != message.imei.0.len() {
            return Err(Error::InvalidImei);
        }
        message.session_status = match SessionStatus::from_u8(try!(cursor.read_u8())) {
            Some(status) => status,
            None => SessionStatus::Unknown,
        };
        message.momsn = try!(cursor.read_u16::<BigEndian>());
        message.mtmsn = try!(cursor.read_u16::<BigEndian>());
        message.time_of_session = UTC.ymd(1970, 1, 1).and_hms(0, 0, 0) +
            Duration::seconds(try!(cursor.read_u32::<BigEndian>()) as i64);

        let payload = match information_elements.remove(&InformationElementType::MobileOriginatedPayload) {
            Some(ie) => ie,
            None => return Err(Error::MissingMobileOriginatedPayload),
        };
        message.payload = payload.into_contents();

        Ok(message)
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
        Ok(())
    }

    /// Returns this message's protocol revision number.
    pub fn protocol_revision_number(&self) -> u8 { self.protocol_revision_number.0 }
    /// Returns this message's IMEI number as a string.
    pub fn imei(&self) -> &str { self.imei.as_str() }
    /// Returns this message's cdr reference number (also called auto id).
    pub fn cdr_reference(&self) -> u32 { self.cdr_reference }
    /// Returns this message's session status.
    pub fn session_status(&self) -> SessionStatus { self.session_status }
    /// Returns this message's mobile originated message number.
    pub fn momsn(&self) -> u16 { self.momsn }
    /// Returns this message's mobile terminated message number.
    pub fn mtmsn(&self) -> u16 { self.mtmsn }
    /// Returns this message's time of session.
    pub fn time_of_session(&self) -> DateTime<UTC> { self.time_of_session }
    /// Returns a reference to this message's payload.
    pub fn payload_ref(&self) -> &Vec<u8> { &self.payload }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::{Cursor, Read};
    use std::str;

    use chrono::{TimeZone, UTC};

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
    fn values() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        assert_eq!(1, message.protocol_revision_number());
        assert_eq!(1894516585, message.cdr_reference());
        assert_eq!("300234063904190", message.imei());
        assert_eq!(SessionStatus::Ok, message.session_status());
        assert_eq!(75, message.momsn());
        assert_eq!(0, message.mtmsn());
        assert_eq!(UTC.ymd(2015, 7, 9).and_hms(18, 15, 8), message.time_of_session());
        assert_eq!("test message from pete", str::from_utf8(message.payload_ref()).unwrap());
    }

    #[test]
    fn write() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        let mut cursor = Cursor::new(Vec::new());
        message.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        let message2 = Message::read_from(cursor).unwrap();
        assert_eq!(message, message2);
    }
}
