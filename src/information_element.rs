//! An information element is a portion of a SBD message.
//!
//! Information elements come after the SBD header. They come in many types, including more
//! header-type information and the actual data payload.

use std::io::Read;

use byteorder::{ReadBytesExt, BigEndian};

use {Error, Result};

/// An information element, or IE.
///
/// These are the building blocks of a SBD message. There are several types, generally divided into
/// MO and MT IE's. This general structure just holds the basic IE data, when then can be converted
/// into a specific type of IE.
#[derive(Debug, Default)]
pub struct InformationElement {
    id: u8,
    length: u16,
    contents: Vec<u8>,
}

impl InformationElement {

    /// Reads an information element from something that implements `Read`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Seek, SeekFrom};
    /// use std::fs::File;
    /// use sbd::information_element::InformationElement;
    /// let mut file = File::open("data/0-mo.sbd").unwrap();
    /// file.seek(SeekFrom::Start(3));
    /// let readable = file.take(28);
    /// InformationElement::read_from(readable).unwrap();
    /// ```
    pub fn read_from<R: Read>(mut readable: R) -> Result<InformationElement> {
        let mut information_element: InformationElement = Default::default();
        information_element.id = try!(readable.read_u8());
        information_element.length = try!(readable.read_u16::<BigEndian>());
        let content_length = information_element.length - 3;
        let bytes_read = try!(readable.take(content_length as u64)
                              .read_to_end(&mut information_element.contents));
        assert!(!(bytes_read > content_length as usize));
        if bytes_read < content_length as usize {
            return Err(Error::Undersized(bytes_read + 3));
        }
        Ok(information_element)
    }

    /// Returns the length of the information element.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::{Seek, SeekFrom};
    /// # use std::fs::File;
    /// # use sbd::information_element::InformationElement;
    /// # let mut file = File::open("data/0-mo.sbd").unwrap();
    /// # file.seek(SeekFrom::Start(3));
    /// # let readable = file.take(28);
    /// let information_element = InformationElement::read_from(readable).unwrap();
    /// assert_eq!(28, information_element.len());
    /// ```
    pub fn len(&self) -> u16 {
        self.length
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
        let readable = file.take(28);
        InformationElement::read_from(readable).unwrap();
    }

    #[test]
    fn len() {
        let mut file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3)).unwrap();
        let readable = file.take(28);
        let ie = InformationElement::read_from(readable).unwrap();
        assert_eq!(28, ie.len());
    }

    #[test]
    fn undersized() {
        let mut file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3)).unwrap();
        let readable = file.take(27);
        assert!(InformationElement::read_from(readable).is_err());
    }
}
