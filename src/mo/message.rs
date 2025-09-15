use std::{
    cmp::Ordering,
    io::{Cursor, Read, Write},
    path::Path,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chrono::{DateTime, Utc};

use crate::{
    mo::{Header, InformationElement, SessionStatus},
    Error,
};

/// The only valid protocol revision number.
pub const PROTOCOL_REVISION_NUMBER: u8 = 1;

/// A mobile-origined Iridium SBD message.
///
/// `Message`s can be ordered by time of session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    header: Header,
    payload: Vec<u8>,
    information_elements: Vec<InformationElement>,
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
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Message, Error> {
        use std::fs::File;
        let file = File::open(path)?;
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
    pub fn read_from<R: Read>(mut read: R) -> Result<Message, Error> {
        let protocol_revision_number = read.read_u8()?;
        if protocol_revision_number != PROTOCOL_REVISION_NUMBER {
            return Err(Error::InvalidProtocolRevisionNumber(
                protocol_revision_number,
            ));
        }
        let overall_message_length = read.read_u16::<BigEndian>()?;
        let mut message = vec![0; overall_message_length as usize];
        read.read_exact(&mut message)?;

        let mut cursor = Cursor::new(message);
        let mut information_elements = Vec::new();
        while cursor.position() < u64::from(overall_message_length) {
            information_elements.push(InformationElement::read_from(&mut cursor)?);
        }

        Message::new(information_elements)
    }

    /// Creates a new message from information elements.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate chrono;
    /// # extern crate sbd;
    /// # fn main() {
    /// use chrono::{Utc, TimeZone};
    /// use sbd::mo::{InformationElement, Header, SessionStatus, Message};
    /// let header = InformationElement::Header(Header {
    ///     auto_id: 1,
    ///     imei: [0; 15],
    ///     session_status: SessionStatus::Ok,
    ///     momsn: 1,
    ///     mtmsn: 0,
    ///     time_of_session: Utc.ymd(2017, 10, 1).and_hms(0, 0, 0),
    /// });
    /// let payload = InformationElement::Payload(Vec::new());
    /// let message = Message::new(vec![header, payload]);
    /// # }
    /// ```
    pub fn new<I: IntoIterator<Item = InformationElement>>(iter: I) -> Result<Message, Error> {
        let mut header = None;
        let mut payload = None;
        let mut information_elements = Vec::new();
        for information_element in iter {
            match information_element {
                InformationElement::Header(h) => {
                    if let Some(header) = header {
                        return Err(Error::TwoHeaders(h, header));
                    } else {
                        header = Some(h);
                    }
                }
                InformationElement::Payload(p) => {
                    if let Some(payload) = payload {
                        return Err(Error::TwoPayloads(p, payload));
                    } else {
                        payload = Some(p);
                    }
                }
                ie => information_elements.push(ie),
            }
        }
        Ok(Message {
            header: header.ok_or(Error::NoHeader)?,
            payload: payload.ok_or(Error::NoPayload)?,
            information_elements,
        })
    }

    /// Returns this message's auto id.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::mo::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let auto_id = message.auto_id();
    /// ```
    pub fn auto_id(&self) -> u32 {
        self.header.auto_id
    }

    /// Returns this message's imei as a string.
    ///
    /// # Panics
    ///
    /// Panics if the IMEI number is not valid utf8. The specification says that IMEIs should be
    /// ascii numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::mo::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let imei = message.imei();
    /// ```
    pub fn imei(&self) -> &str {
        use std::str;
        str::from_utf8(&self.header.imei).expect("IMEI numbers are specified to be ascii number")
    }

    /// Returns this message's session status.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::mo::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let session_status = message.session_status();
    /// ```
    pub fn session_status(&self) -> SessionStatus {
        self.header.session_status
    }

    /// Returns this message's mobile originated message sequence number.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::mo::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let momsn = message.momsn();
    /// ```
    pub fn momsn(&self) -> u16 {
        self.header.momsn
    }

    /// Returns this message's mobile terminated message sequence number.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::mo::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let mtmsn = message.mtmsn();
    /// ```
    pub fn mtmsn(&self) -> u16 {
        self.header.mtmsn
    }

    /// Returns this message's time of session.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::mo::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let time_of_session = message.time_of_session();
    /// ```
    pub fn time_of_session(&self) -> DateTime<Utc> {
        self.header.time_of_session
    }

    /// Returns this message's payload.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::mo::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let payload = message.payload();
    /// ```
    pub fn payload(&self) -> &Vec<u8> {
        &self.payload
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
    pub fn write_to<W: Write>(&self, mut write: W) -> Result<(), Error> {
        let header = InformationElement::from(self.header);
        let payload = InformationElement::from(self.payload.clone());
        let overall_message_length = header.len()
            + payload.len()
            + self
                .information_elements
                .iter()
                .map(|ie| ie.len())
                .sum::<usize>();
        if overall_message_length > u16::MAX as usize {
            return Err(Error::OverallMessageLength(overall_message_length));
        }

        write.write_u8(PROTOCOL_REVISION_NUMBER)?;
        write.write_u16::<BigEndian>(overall_message_length as u16)?;
        header.write_to(&mut write)?;
        payload.write_to(&mut write)?;
        for information_element in &self.information_elements {
            information_element.write_to(&mut write)?;
        }
        Ok(())
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time_of_session().cmp(&other.time_of_session())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Cursor, Read},
        str,
    };

    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::mo::Header;

    pub fn header() -> Header {
        Header {
            auto_id: 1,
            imei: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
            session_status: SessionStatus::Ok,
            momsn: 1,
            mtmsn: 0,
            time_of_session: Utc.with_ymd_and_hms(2017, 10, 1, 1, 2, 3).single().unwrap(),
        }
    }

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
    fn no_payload() {
        let header = header();
        assert!(Message::new(vec![header.into()]).is_err());
    }

    #[test]
    fn two_payloads() {
        let header = header();
        let payload = Vec::new();
        assert!(Message::new(vec![header.into(), payload.clone().into(), payload.into()]).is_err());
    }

    #[test]
    fn no_header() {
        assert!(Message::new(vec![vec![].into()]).is_err());
    }

    #[test]
    fn two_headers() {
        let header = header();
        assert!(Message::new(vec![header.into(), header.into()]).is_err());
    }

    #[test]
    fn values() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        assert_eq!(1894516585, message.auto_id());
        assert_eq!("300234063904190", message.imei());
        assert_eq!(SessionStatus::Ok, message.session_status());
        assert_eq!(75, message.momsn());
        assert_eq!(0, message.mtmsn());
        assert_eq!(
            Utc.with_ymd_and_hms(2015, 7, 9, 18, 15, 8)
                .single()
                .unwrap(),
            message.time_of_session()
        );
        assert_eq!(
            "test message from pete",
            str::from_utf8(message.payload()).unwrap()
        );
    }

    #[test]
    fn write() {
        let message = Message::new(vec![
            header().into(),
            vec![1].into(),
            InformationElement::LocationInformation([0; 7]),
        ])
        .unwrap();
        let mut cursor = Cursor::new(Vec::new());
        message.write_to(&mut cursor).unwrap();
        cursor.set_position(0);
        let message2 = Message::read_from(cursor).unwrap();
        assert_eq!(message, message2);
    }

    #[test]
    fn order() {
        let header1 = header();
        let mut header2 = header();
        header2.time_of_session = Utc.with_ymd_and_hms(2010, 6, 11, 0, 0, 0).single().unwrap();
        let message1 = Message::new(vec![header1.into(), Vec::new().into()]).unwrap();
        let message2 = Message::new(vec![header2.into(), Vec::new().into()]).unwrap();
        assert!(message2 < message1);
    }
}
