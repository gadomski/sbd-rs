//! Squirrel away SBD messages and retrieve them later.

mod filesystem;
mod memory;

pub use self::filesystem::Storage as FilesystemStorage;
pub use self::memory::Storage as MemoryStorage;

use Result;
use mo::Message;

/// Basic storage operations.
pub trait Storage {
    /// Place a message into this storage.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sbd::mo::Message;
    /// # use sbd::storage::{Storage, MemoryStorage};
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let mut storage = MemoryStorage::new();
    /// storage.store(&message);
    /// ```
    fn store(&mut self, message: &Message) -> Result<()>;

    /// Retrieve all messages in this storage as a vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sbd::mo::Message;
    /// # use sbd::storage::{Storage, MemoryStorage};
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let mut storage = MemoryStorage::new();
    /// storage.store(&message);
    /// let messages = storage.messages().unwrap();
    /// assert_eq!(vec![message], messages);
    /// ```
    fn messages(&self) -> Result<Vec<Message>>;
}
