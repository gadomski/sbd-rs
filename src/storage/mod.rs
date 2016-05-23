//! SBD message storage.
//!
//! We can store messages in one of several backends, provided as submodules here. Storages
//! implement the `Storage` trait.

mod filesystem;
mod memory;

pub use self::filesystem::Storage as FilesystemStorage;
pub use self::memory::Storage as MemoryStorage;

use Result;
use mo::Message;

/// Trait for all backend SBD storages.
pub trait Storage {
    /// Stores message in this storage.
    fn store(&mut self, message: &Message) -> Result<()>;
}
