//! Squirrel away SBD messages and retrieve them later.

mod filesystem;
mod memory;

pub use self::filesystem::Storage as FilesystemStorage;
pub use self::memory::Storage as MemoryStorage;

use Result;
use mo::Message;

/// Basic storage operations.
pub trait Storage {
    /// Stores a message, consuming it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sbd::mo::Message;
    /// # use sbd::storage::{Storage, MemoryStorage};
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let mut storage = MemoryStorage::new();
    /// storage.store(message);
    /// ```
    fn store(&mut self, message: Message) -> Result<()>;

    /// Retrieves all messages in this storage as a vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sbd::mo::Message;
    /// # use sbd::storage::{Storage, MemoryStorage};
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let mut storage = MemoryStorage::new();
    /// storage.store(message.clone());
    /// let messages = storage.messages().unwrap();
    /// assert_eq!(vec![message], messages);
    /// ```
    fn messages(&self) -> Result<Vec<Message>>;

    /// Retrieves all messages for a given IMEI.
    ///
    /// The default implementation just filters the vector provided by `messages`, but some
    /// implementors could be more intelligent.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sbd::mo::Message;
    /// # use sbd::storage::{Storage, MemoryStorage};
    /// let message = Message::from_path("data/0-mo.sbd").unwrap();
    /// let mut storage = MemoryStorage::new();
    /// storage.store(message.clone());
    /// let messages = storage.messages_from_imei("300234063904190").unwrap();
    /// assert_eq!(vec![message], messages);
    /// let messages = storage.messages_from_imei("300234063904191").unwrap();
    /// assert!(messages.is_empty());
    /// ```
    fn messages_from_imei(&self, imei: &str) -> Result<Vec<Message>> {
        self.messages().map(|mut v| {
            v.retain(|m| m.imei() == imei);
            v
        })
    }
}
