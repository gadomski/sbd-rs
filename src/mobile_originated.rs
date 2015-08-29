//! Handling for mobile originated (MO) messages.
//!
//! SBD messages really can come in two forms, mobile-originated and mobile-terminated. This module
//! provides functionality for the MO subset of messages.

use std::io::Read;
use std::path::Path;

use byteorder::{ReadBytesExt, BigEndian};
use num::traits::FromPrimitive;

use {Error, Result};
use information_element::{SessionStatus, InformationElementType};
use message::Message;

/// A mobile originated (MO) message;
#[derive(Debug)]
pub struct MobileOriginated {
    cdr_reference: u32,
    imei: [u8; 15],
    session_status: SessionStatus,
    momsn: u16,
    mtmsn: u16,
    time: u32,
    payload: Vec<u8>,
}

impl Default for MobileOriginated {
    fn default() -> MobileOriginated {
        MobileOriginated {
            cdr_reference: 0,
            imei: [0; 15],
            session_status: SessionStatus::Unknown,
            momsn: 0,
            mtmsn: 0,
            time: 0,
            payload: Vec::new(),
        }
    }
}

impl MobileOriginated {

    /// Reads a mobile originated message from a `Path`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::mobile_originated::MobileOriginated;
    /// MobileOriginated::from_path("data/0-mo.sbd");
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<MobileOriginated> {
        Ok(try!(try!(Message::from_path(path)).into_mobile_originated()))
    }

    /// Returns the cdr reference, also called the auto ID.
    ///
    /// This is a unique value given each call data record.
    pub fn cdr_reference(&self) -> u32 {
        self.cdr_reference
    }

    /// Returns the IMEI number.
    ///
    /// This is unique to each unit, and is always 15 characters long.
    pub fn imei(&self) -> &[u8] {
        &self.imei
    }

    /// Returns the status of the mobile originated session.
    pub fn session_status(&self) -> SessionStatus {
        self.session_status
    }

    /// Returns the mobile originated message sequence number.
    ///
    /// This value is set by the IMEI, and is incremented for every successful SBD session.
    pub fn momsn(&self) -> u16 {
        self.momsn
    }

    /// Returns the mobile terminated message squence number.
    ///
    /// This value is set by the Gateway when the message is queued for delivery. The MTMSN is
    /// tranferred to the IMEI as part of the MT payload transfer, regardless of the session
    /// success.
    pub fn mtmsn(&self) -> u16 {
        self.mtmsn
    }

    /// Returns the UTC timestamp in the form of an epoch integer.
    ///
    /// This is the time of the session between the IMEI and the Gateway, in seconds since the
    /// start of the epoch: 1/1/1970 00:00:00.
    pub fn time(&self) -> u32 {
        self.time
    }

    /// Returns the MO payload.
    pub fn payload(&self) -> &Vec<u8> {
        &self.payload
    }
}

impl Message {

    /// Convert a message into a mobile originated message.
    ///
    /// Returns an error if this message isn't really mobile originated.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::message::Message;
    /// use sbd::mobile_originated;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// message.into_mobile_originated().unwrap();
    /// ```
    pub fn into_mobile_originated(self) -> Result<MobileOriginated> {
        let mut information_elements = self.into_information_elements();

        // TODO should we fail if the payload is absent?
        let header = match information_elements.remove(&InformationElementType::MobileOriginatedHeader) {
            Some(header) => header,
            None => return Err(Error::NoMobileOriginatedHeader),
        };
        let payload = match information_elements.remove(&InformationElementType::MobileOriginatedPayload) {
            Some(payload) => payload,
            None => return Err(Error::NoMobileOriginatedPayload),
        };

        let mut readable = header.as_contents_reader();
        let mut mobile_originated: MobileOriginated = Default::default();
        mobile_originated.cdr_reference = try!(readable.read_u32::<BigEndian>());
        let bytes_read = try!(readable.read(&mut mobile_originated.imei));
        if bytes_read != mobile_originated.imei.len() {
            return Err(Error::Undersized(bytes_read));
        }
        mobile_originated.session_status = match SessionStatus::from_u8(try!(readable.read_u8())) {
            Some(status) => status,
            None => SessionStatus::Unknown,
        };
        mobile_originated.momsn = try!(readable.read_u16::<BigEndian>());
        mobile_originated.mtmsn = try!(readable.read_u16::<BigEndian>());
        mobile_originated.time = try!(readable.read_u32::<BigEndian>());

        mobile_originated.payload = payload.into_contents();

        Ok(mobile_originated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str;

    use information_element::SessionStatus;
    use message::Message;

    #[test]
    fn into_mobile_originated() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        message.into_mobile_originated().unwrap();
    }

    #[test]
    fn from_path() {
        MobileOriginated::from_path("data/0-mo.sbd").unwrap();
    }

    #[test]
    fn mo_header_values() {
        let mo = MobileOriginated::from_path("data/0-mo.sbd").unwrap();
        assert_eq!(1894516585, mo.cdr_reference());
        assert_eq!("300234063904190", str::from_utf8(mo.imei()).unwrap());
        assert_eq!(SessionStatus::Ok, mo.session_status());
        assert_eq!(75, mo.momsn());
        assert_eq!(0, mo.mtmsn());
        assert_eq!(1436465708, mo.time());
    }

    #[test]
    fn mo_payload() {
        let mo = MobileOriginated::from_path("data/0-mo.sbd").unwrap();
        assert_eq!("test message from pete", str::from_utf8(mo.payload()).unwrap());
    }
}
