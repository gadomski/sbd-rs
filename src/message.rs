//! Generic message handling.
//!
//! This module provides the ability to read SBD messages from and write SBD messages to streams.

use std::path::Path;

use Result;

/// An SBD message.
pub struct Message;

impl Message {

    /// Reads in a message from a file path.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::message::Message;
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Message> {
        Ok(Message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_from_path() {
        Message::from_path("data/0-mo.sbd").unwrap();
    }
}
