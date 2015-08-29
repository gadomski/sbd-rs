//! An information element is a portion of a SBD message.
//!
//! Information elements come after the SBD header. They come in many types, including more
//! header-type information and the actual data payload.

use std::io::{Cursor, Read};

use byteorder::{ReadBytesExt, BigEndian};
use num::traits::FromPrimitive;

use {Error, Result};

const INFORMATION_ELEMENT_HEADER_LENGTH: u16 = 3;

/// Indicates the success or failure of the SBD session.
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

/// Enum to name the information element ids.
enum_from_primitive! {
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum InformationElementType {
    MobileOriginatedHeader = 0x01,
    MobileOriginatedPayload = 0x02,
    MobileOriginatedLocationInformation = 0x03,
    MobileTerminatedHeader = 0x41,
    MobileTerminatedPayload = 0x42,
    MobileTerminatedConfirmationMessage = 0x44,
    Unknown,
}
}

impl Default for InformationElementType {
    fn default() -> InformationElementType {
        InformationElementType::Unknown
    }
}

/// An information element, or IE.
///
/// These are the building blocks of a SBD message. There are several types, generally divided into
/// MO and MT IE's. This general structure just holds the basic IE data, when then can be converted
/// into a specific type of IE.
#[derive(Debug, Default)]
pub struct InformationElement {
    id: InformationElementType,
    length: u16,
    contents: Vec<u8>,
}

impl InformationElement {

    /// Reads an information element from something that implements `Read`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Read, Seek, SeekFrom};
    /// use std::fs::File;
    /// use sbd::information_element::InformationElement;
    /// let mut file = File::open("data/0-mo.sbd").unwrap();
    /// file.seek(SeekFrom::Start(3)).unwrap();
    /// let readable = file.take(31);
    /// InformationElement::read_from(readable).unwrap();
    /// ```
    pub fn read_from<R: Read>(mut readable: R) -> Result<InformationElement> {
        let mut information_element: InformationElement = Default::default();
        information_element.id = match InformationElementType::from_u8(try!(readable.read_u8())) {
            Some(ietype) => ietype,
            None => InformationElementType::Unknown,
        };
        information_element.length = try!(readable.read_u16::<BigEndian>());
        let bytes_read = try!(readable.take(information_element.length as u64)
                              .read_to_end(&mut information_element.contents));
        assert!(!(bytes_read > information_element.length as usize));
        if bytes_read < information_element.length as usize {
            return Err(Error::Undersized(bytes_read + 3));
        }
        Ok(information_element)
    }

    /// Returns the length of the information element.
    ///
    /// This is not the same as the internal length, but is rather the length of the contents plus
    /// the length of the IE header.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::{Read, Seek, SeekFrom};
    /// # use std::fs::File;
    /// # use sbd::information_element::InformationElement;
    /// # let mut file = File::open("data/0-mo.sbd").unwrap();
    /// # file.seek(SeekFrom::Start(3)).unwrap();
    /// # let readable = file.take(31);
    /// let information_element = InformationElement::read_from(readable).unwrap();
    /// assert_eq!(31, information_element.len());
    /// ```
    pub fn len(&self) -> u16 {
        self.length + INFORMATION_ELEMENT_HEADER_LENGTH
    }

    /// Returns the id of the information element.
    pub fn id(&self) -> InformationElementType {
        self.id
    }

    /// Returns a object that can `Read` over the contents of this information element.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::{Read, Seek, SeekFrom};
    /// # use std::fs::File;
    /// # use sbd::information_element::InformationElement;
    /// # let mut file = File::open("data/0-mo.sbd").unwrap();
    /// # file.seek(SeekFrom::Start(3)).unwrap();
    /// # let readable = file.take(31);
    /// let information_element = InformationElement::read_from(readable).unwrap();
    /// let mut readable = information_element.as_contents_reader();
    /// let mut buffer: Vec<u8> = Vec::new();
    /// readable.read_to_end(&mut buffer);
    /// ```
    pub fn as_contents_reader<'a>(&self) -> Cursor<&[u8]> {
        Cursor::new(&self.contents[..])
    }

    /// Return a reference to this information element's contents.
    pub fn contents_ref(&self) -> &Vec<u8> {
        &self.contents
    }

    /// Convert this information element into its contents.
    ///
    /// This consumes the information element.
    pub fn into_contents(self) -> Vec<u8> {
        self.contents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{Read, Seek, SeekFrom};
    use std::fs::File;

    #[test]
    fn read_from() {
        let mut file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3)).unwrap();
        let readable = file.take(31);
        InformationElement::read_from(readable).unwrap();
    }

    #[test]
    fn len() {
        let mut file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3)).unwrap();
        let readable = file.take(31);
        let ie = InformationElement::read_from(readable).unwrap();
        assert_eq!(31, ie.len());
    }

    #[test]
    fn undersized() {
        let mut file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3)).unwrap();
        let readable = file.take(30);
        assert!(InformationElement::read_from(readable).is_err());
    }
}
