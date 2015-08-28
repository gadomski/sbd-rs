//! Handling for mobile originated (MO) messages.
//!
//! SBD messages really can come in two forms, mobile-originated and mobile-terminated. This module
//! provides functionality for the MO subset of messages.

use std::path::Path;

use Result;
use message::Message;

/// A mobile originated (MO) message;
pub struct MobileOriginated;

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
        Ok(MobileOriginated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
