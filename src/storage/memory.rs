//! Store SBD messages in memory.
//!
//! Useful primarily for testing.

use Result;
use mo::Message;
use storage;

/// A simple storage backend that saves the messages in memory.
#[derive(Debug)]
pub struct Storage {
    messages: Vec<Message>,
}

impl Storage {
    /// Creates a new memory storage.
    ///
    /// # Examples
    ///
    /// ```
    /// let storage = sbd::storage::MemoryStorage::new();
    /// ```
    pub fn new() -> Storage {
        Storage { messages: Vec::new() }
    }

    /// Returns a reference to the underlying message vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::storage::{MemoryStorage, Storage};
    /// let mut storage = MemoryStorage::new();
    /// assert!(storage.messages().is_empty());
    /// storage.store(&sbd::mo::Message::from_path("data/0-mo.sbd").unwrap());
    /// assert_eq!(1, storage.messages().len());
    /// ```
    pub fn messages(&self) -> &Vec<Message> {
        &self.messages
    }
}

impl storage::Storage for Storage {
    fn store(&mut self, message: &Message) -> Result<()> {
        self.messages.push((*message).clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use mo::Message;
    use storage::Storage as StorageTrait;

    #[test]
    fn store_memory() {
        let mut storage = Storage::new();
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        storage.store(&message).unwrap();
        let ref stored_message = storage.messages()[0];
        assert_eq!(&message, stored_message);
    }
}
