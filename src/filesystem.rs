//! Manage SBD messages stored on the fileystem.
//!
//! Messages are stored in a directory hierarchy under a single root directory. Message storage and
//! retrieval are managed by a `Storage` object, which is configured for a single root directory.

use std::path::Path;

/// A structure for managing storing and retriving SBD messages on a filesystem.
pub struct Storage;

impl Storage {
    /// Creates a new storage manager for a given root directory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sbd::filesystem::Storage;
    /// let storage = Storage::new("/var/iridium");
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Storage {
        Storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempdir::TempDir;

    #[test]
    fn new_storage() {
        Storage::new(TempDir::new("").unwrap().path());
    }
}
