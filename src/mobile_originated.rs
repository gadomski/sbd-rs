//! Handling for mobile originated (MO) messages.
//!
//! SBD messages really can come in two forms, mobile-originated and mobile-terminated. This module
//! provides functionality for the MO subset of messages.

use std::path::Path;

use Result;
use information_element::SessionStatus;
use message::Message;

/// A mobile originated (MO) message;
#[derive(Debug)]
pub struct MobileOriginated {
    imei: [u8; 15],
}

impl Default for MobileOriginated {
    fn default() -> MobileOriginated {
        MobileOriginated {
            imei: [0; 15]
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
        0
    }

    /// Returns the IMEI number.
    ///
    /// This is unique to each unit, and is always 15 characters long.
    pub fn imei(&self) -> &[u8] {
        &self.imei
    }

    /// Returns the status of the mobile originated session.
    pub fn session_status(&self) -> SessionStatus {
        SessionStatus::Ok
    }

    /// Returns the mobile originated message sequence number.
    ///
    /// This value is set by the IMEI, and is incremented for every successful SBD session.
    pub fn momsn(&self) -> u16 {
        0
    }

    /// Returns the mobile terminated message squence number.
    ///
    /// This value is set by the Gateway when the message is queued for delivery. The MTMSN is
    /// tranferred to the IMEI as part of the MT payload transfer, regardless of the session
    /// success.
    pub fn mtmsn(&self) -> u16 {
        0
    }

    /// Returns the UTC timestamp in the form of an epoch integer.
    ///
    /// This is the time of the session between the IMEI and the Gateway, in seconds since the
    /// start of the epoch: 1/1/1970 00:00:00.
    pub fn time(&self) -> u32 {
        0
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
        let mut mobile_originated: MobileOriginated = Default::default();
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
        assert_eq!(1, mo.cdr_reference());
        assert_eq!("1234", str::from_utf8(mo.imei()).unwrap());
        assert_eq!(SessionStatus::Ok, mo.session_status());
        assert_eq!(123, mo.momsn());
        assert_eq!(123, mo.mtmsn());
        assert_eq!(123, mo.time());
    }
}
