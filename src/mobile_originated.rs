//! Handling for mobile originated (MO) messages.
//!
//! SBD messages really can come in two forms, mobile-originated and mobile-terminated. This module
//! provides functionality for the MO subset of messages.

use Result;
use message::Message;

/// A mobile originated (MO) message;
pub struct MobileOriginated;

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
    fn from_message() {
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        let mo = message.into_mobile_originated().unwrap();
    }
}
