//! Module for reading and writing Mobile-Originated (MO) SBD messages.
//!
//! Though messages technically come in two flavors, mobile originated and mobile terminated, we
//! only handle mobile originated messages in this library.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::str;
use std::path::Path;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chrono::{DateTime, Duration, TimeZone, UTC};

use {Error, Result};
use information_element::{InformationElement, InformationElementType};

const INFORMATION_ELEMENT_HEADER_LENGTH: u16 = 3;
const MOBILE_ORIGINATED_HEADER_LENGTH: u16 = 28;
const ASCII_ZERO: u8 = 48;

/// The protocol number of an SBD message.
///
/// At this point, this can *only* be one.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
struct ProtocolRevisionNumber(u8);

impl ProtocolRevisionNumber {
    /// Returns true if this is a valid protocol revision number.
    pub fn valid(&self) -> bool {
        self.0 == 1
    }
}

/// The modem IMEI identifier.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Imei([u8; 15]);

impl Default for Imei {
    fn default() -> Imei {
        Imei([ASCII_ZERO; 15])
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
///
/// The descriptions for these codes are taken directly from the `DirectIP` documentation.
#[derive(Copy, Clone, Debug, Eq, PartialEq, RustcEncodable)]
pub enum SessionStatus {
    /// The SBD session completed successfully.
    Ok = 0,
    /// The MO message transfer, if any, was successful. The MT message queued at the GSS is too
    /// large to be transferred within a single SBD session.
    OkMobileTerminatedTooLarge = 1,
    /// The MO message transfer, if any, was successful. The reported location was determined to be
    /// of unacceptable quality. This value is only applicable to IMEIs using SBD protocol revision
    /// 1.
    OkLocationUnacceptableQuality = 2,
    /// The SBD session timed out before session completion.
    Timeout = 10,
    /// The MO message being transferred by the IMEI is too large to be transerred within a single
    /// SBD session.
    MobileOriginatedTooLarge = 12,
    /// An RF link loss ocurred during the SBD session.
    RFLinkLoss = 13,
    /// An IMEI protocol anomaly occurred during SBD session.
    IMEIProtocolAnomaly = 14,
    /// The IMEI is prohibited from accessing the GSS.
    Prohibited = 15,
    /// Unknown session status code.
    Unknown,
}

impl Default for SessionStatus {
    fn default() -> SessionStatus {
        SessionStatus::Unknown
    }
}

impl From<u8> for SessionStatus {
    fn from(n: u8) -> Self {
        match n {
            0 => SessionStatus::Ok,
            1 => SessionStatus::OkMobileTerminatedTooLarge,
            2 => SessionStatus::OkLocationUnacceptableQuality,
            10 => SessionStatus::Timeout,
            12 => SessionStatus::MobileOriginatedTooLarge,
            13 => SessionStatus::RFLinkLoss,
            14 => SessionStatus::IMEIProtocolAnomaly,
            15 => SessionStatus::Prohibited,
            _ => SessionStatus::Unknown,
        }
    }
}

/// A mobile-origined Iridium SBD message.
///
/// `Message`s can be ordered by time of session.
#[derive(Clone, Debug, Eq, PartialEq)]
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
            protocol_revision_number: ProtocolRevisionNumber(1),
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

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time_of_session.cmp(&other.time_of_session)
    }
}

impl Message {
    /// Reads in a message from a file path.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::mo::Message;
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
    /// use sbd::mo::Message;
    /// let mut file = File::open("data/0-mo.sbd").unwrap();
    /// let message = Message::read_from(file).unwrap();
    /// ```
    pub fn read_from<R: Read>(mut readable: R) -> Result<Message> {
        let mut message: Message = Default::default();

        message.protocol_revision_number = ProtocolRevisionNumber(try!(readable.read_u8()));
        if !message.protocol_revision_number.valid() {
            return Err(Error::InvalidProtocolRevisionNumber(message.protocol_revision_number.0));
        }
        let overall_message_length = try!(readable.read_u16::<BigEndian>());

        let mut information_elements: HashMap<InformationElementType, InformationElement> =
            HashMap::new();
        let mut bytes_read = 0u16;
        loop {
            let ie = match InformationElement::read_from(&mut readable) {
                Ok(ie) => ie,
                Err(e) => return Err(e),
            };
            bytes_read += ie.len();
            information_elements.insert(ie.id(), ie);
            if bytes_read >= overall_message_length {
                break;
            }
        }

        if try!(readable.take(1).read_to_end(&mut Vec::new())) != 0 {
            return Err(Error::Oversized);
        }

        let header =
            match information_elements.remove(&InformationElementType::MobileOriginatedHeader) {
                Some(ie) => ie,
                None => return Err(Error::MissingMobileOriginatedHeader),
            };
        let mut cursor = &mut Cursor::new(header.contents_ref());
        message.cdr_reference = try!(cursor.read_u32::<BigEndian>());
        let bytes_read = try!(cursor.take(message.imei.0.len() as u64).read(&mut message.imei.0));
        if bytes_read != message.imei.0.len() {
            return Err(Error::InvalidImei);
        }
        message.session_status = SessionStatus::from(try!(cursor.read_u8()));
        message.momsn = try!(cursor.read_u16::<BigEndian>());
        message.mtmsn = try!(cursor.read_u16::<BigEndian>());
        message.time_of_session = UTC.ymd(1970, 1, 1).and_hms(0, 0, 0) +
                                  Duration::seconds(try!(cursor.read_u32::<BigEndian>()) as i64);

        let payload =
            match information_elements.remove(&InformationElementType::MobileOriginatedPayload) {
                Some(ie) => ie,
                None => return Err(Error::MissingMobileOriginatedPayload),
            };
        message.payload = payload.into_contents();

        if !information_elements.is_empty() {
            return Err(Error::UnhandledInformationElements(information_elements));
        }

        Ok(message)
    }

    /// Write this message back to a object that can `Write`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use sbd::mo::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let mut cursor = Cursor::new(Vec::new());
    /// message.write_to(&mut cursor);
    /// ```
    pub fn write_to<W: Write>(&self, w: &mut W) -> Result<()> {
        try!(w.write_u8(self.protocol_revision_number.0));
        try!(w.write_u16::<BigEndian>(self.overall_message_length()));
        try!(w.write_u8(InformationElementType::MobileOriginatedHeader as u8));
        try!(w.write_u16::<BigEndian>(MOBILE_ORIGINATED_HEADER_LENGTH));
        try!(w.write_u32::<BigEndian>(self.cdr_reference));
        try!(w.write_all(&self.imei.0));
        try!(w.write_u8(self.session_status as u8));
        try!(w.write_u16::<BigEndian>(self.momsn));
        try!(w.write_u16::<BigEndian>(self.mtmsn));
        try!(w.write_u32::<BigEndian>(self.time_of_session.timestamp() as u32));
        try!(w.write_u8(InformationElementType::MobileOriginatedPayload as u8));
        // TODO can we check to make sure the payload is of appropriate size?
        try!(w.write_u16::<BigEndian>(self.payload.len() as u16));
        try!(w.write_all(&self.payload[..]));
        Ok(())
    }

    /// Returns this message's protocol revision number.
    pub fn protocol_revision_number(&self) -> u8 {
        self.protocol_revision_number.0
    }
    /// Returns this message's IMEI number as a string.
    pub fn imei(&self) -> &str {
        self.imei.as_str()
    }
    /// Returns this message's cdr reference number (also called auto id).
    pub fn cdr_reference(&self) -> u32 {
        self.cdr_reference
    }
    /// Returns this message's session status.
    pub fn session_status(&self) -> SessionStatus {
        self.session_status
    }
    /// Returns this message's mobile originated message number.
    pub fn momsn(&self) -> u16 {
        self.momsn
    }
    /// Returns this message's mobile terminated message number.
    pub fn mtmsn(&self) -> u16 {
        self.mtmsn
    }
    /// Returns this message's time of session.
    pub fn time_of_session(&self) -> DateTime<UTC> {
        self.time_of_session
    }
    /// Returns a reference to this message's payload.
    pub fn payload_ref(&self) -> &Vec<u8> {
        &self.payload
    }
    /// Returns this message's payload as a str.
    pub fn payload_str(&self) -> Result<&str> {
        str::from_utf8(&self.payload).map_err(Error::from)
    }

    /// Returns the overall message length, as is contained in the message's header.
    ///
    /// The whole message is actually three bytes longer than this value, thanks to the message
    /// header itself.
    fn overall_message_length(&self) -> u16 {
        INFORMATION_ELEMENT_HEADER_LENGTH + MOBILE_ORIGINATED_HEADER_LENGTH +
        INFORMATION_ELEMENT_HEADER_LENGTH + self.payload.len() as u16
    }
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
        assert_eq!(UTC.ymd(2015, 7, 9).and_hms(18, 15, 8),
                   message.time_of_session());
        assert_eq!("test message from pete",
                   str::from_utf8(message.payload_ref()).unwrap());
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

    #[test]
    fn write_default() {
        let message: Message = Default::default();
        let mut cursor = Cursor::new(Vec::new());
        message.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        let message2 = Message::read_from(cursor).unwrap();
        assert_eq!(message, message2);
    }

    #[test]
    fn order() {
        let default: Message = Default::default();
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        assert!(message > default);
    }

    #[test]
    fn payload_str() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        assert_eq!("test message from pete", message.payload_str().unwrap());
    }
}
