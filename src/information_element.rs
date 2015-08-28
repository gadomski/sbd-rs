//! An information element is a portion of a SBD message.
//!
//! Information elements come after the SBD header. They come in many types, including more
//! header-type information and the actual data payload.

use std::io::Read;

use Result;

/// An information element, or IE.
///
/// These are the building blocks of a SBD message. There are several types, generally divided into
/// MO and MT IE's. This general structure just holds the basic IE data, when then can be converted
/// into a specific type of IE.
#[derive(Debug)]
pub struct InformationElement;

impl InformationElement {

    /// Reads an information element from something that implements `Read`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Seek, SeekFrom};
    /// use std::fs::File;
    /// use sbd::information_element::InformationElement;
    /// let file = File::open("data/0-mo.sbd").unwrap();
    /// file.seek(SeekFrom::Start(3));
    /// let readable = file.take(28);
    /// InformationElement::read_from(readable).unwrap();
    /// ```
    pub fn read_from<R: Read>(readable: R) -> Result<InformationElement> {
        Ok(InformationElement)
    }

    /// Returns the length of the information element.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::{Seek, SeekFrom};
    /// # use std::fs::File;
    /// # use sbd::information_element::InformationElement;
    /// # let file = File::open("data/0-mo.sbd").unwrap();
    /// # file.seek(SeekFrom::Start(3));
    /// # let readable = file.take(28);
    /// let information_element = InformationElement::read_from(readable).unwrap();
    /// assert_eq!(28, information_element.len());
    /// ```
    pub fn len(&self) -> u16 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{Seek, SeekFrom};
    use std::fs::File;

    #[test]
    fn read_from() {
        let file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3));
        let readable = file.take(28);
        InformationElement::read_from(readable).unwrap();
    }

    #[test]
    fn len() {
        let file = File::open("data/0-mo.sbd").unwrap();
        file.seek(SeekFrom::Start(3));
        let readable = file.take(28);
        let ie = InformationElement::read_from(readable).unwrap();
        assert_eq!(28, ie.len());
    }
}
