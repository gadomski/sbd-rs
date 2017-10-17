//! An information element is a portion of a SBD message.
//!
//! Information elements come after the SBD header. They come in many types,
//! including more header-type information and the actual data payload.


use {Error, Result};

use byteorder::{BigEndian, ReadBytesExt};
use std::io::Read;

const INFORMATION_ELEMENT_HEADER_LENGTH: u16 = 3;

/// Enum to name the information element ids.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum InformationElementType {
    /// The header of the mobile originated message.
    MobileOriginatedHeader = 0x01,
    /// The actual mobile originated payload.
    MobileOriginatedPayload = 0x02,
    /// An estimate of the originating IMEI's location.
    ///
    /// An optional IE.
    MobileOriginatedLocationInformation = 0x03,
    /// The header of the mobile terminated message.
    MobileTerminatedHeader = 0x41,
    /// The actual mobile termianted payload.
    MobileTerminatedPayload = 0x42,
    /// A confirmation message.
    MobileTerminatedConfirmationMessage = 0x44,
    /// An unknown IMEI.
    Unknown,
}

impl From<u8> for InformationElementType {
    fn from(n: u8) -> Self {
        match n {
            0x01 => InformationElementType::MobileOriginatedHeader,
            0x02 => InformationElementType::MobileOriginatedPayload,
            0x03 => InformationElementType::MobileOriginatedLocationInformation,
            0x41 => InformationElementType::MobileTerminatedHeader,
            0x42 => InformationElementType::MobileTerminatedPayload,
            0x44 => InformationElementType::MobileTerminatedConfirmationMessage,
            _ => InformationElementType::Unknown,
        }
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
    pub fn read_from<R: Read>(mut readable: R) -> Result<InformationElement> {
        let mut information_element: InformationElement = Default::default();
        information_element.id = InformationElementType::from(readable.read_u8()?);
        information_element.length = readable.read_u16::<BigEndian>()?;
        let bytes_read = readable
            .take(u64::from(information_element.length))
            .read_to_end(&mut information_element.contents)?;
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
    pub fn len(&self) -> u16 {
        self.length + INFORMATION_ELEMENT_HEADER_LENGTH
    }

    /// Returns the id of the information element.
    pub fn id(&self) -> InformationElementType {
        self.id
    }
    /// Returns a reference to this information element's contents.
    pub fn contents_ref(&self) -> &[u8] {
        &self.contents[..]
    }
    /// Converts this information element into its contents.
    pub fn into_contents(self) -> Vec<u8> {
        self.contents
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    use std::io::{Read, Seek, SeekFrom};

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
