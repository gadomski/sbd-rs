//! Store SBD messages in memory.
//!
//! Useful primarily for testing.

use crate::mo::Message;
use crate::storage;

/// A simple storage backend that saves the messages in memory.
#[derive(Debug, Default)]
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
        Storage {
            messages: Vec::new(),
        }
    }
}

impl storage::Storage for Storage {
    fn store(&mut self, message: Message) -> Result<(), ::failure::Error> {
        self.messages.push(message);
        Ok(())
    }

    fn messages(&self) -> Result<Vec<Message>, ::failure::Error> {
        Ok(self.messages.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::mo::Message;
    use crate::storage::Storage as StorageTrait;

    #[test]
    fn store() {
        let mut storage = Storage::new();
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        storage.store(message.clone()).unwrap();
        let stored_message = &storage.messages().unwrap()[0];
        assert_eq!(&message, stored_message);
    }
}
