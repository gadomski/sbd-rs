//! Store SBD messages on the filesystem.

use std::fs;
use std::path::Path;

use {Result, Error};
use mo::Message;
use storage;

const SBD_EXTENSION: &'static str = "sbd";

/// A structure for managing storing and retriving SBD messages on a filesystem.
///
/// Messages are stored in a directory hierarchy under a single root directory.
/// Message storage and retrieval are managed by a `Storage` object, which is
/// configured for a single root directory.
#[derive(Debug)]
pub struct Storage<P: AsRef<Path>> {
    root: P,
}

impl<P: AsRef<Path>> Storage<P> {
    /// Opens a new storage for a given directory.
    ///
    /// # Errors
    ///
    /// If the directory does not exist, returns an `NotADirectory` error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::storage::FilesystemStorage;
    /// let storage = FilesystemStorage::open("data").unwrap();
    /// assert!(FilesystemStorage::open("not/a/directory").is_err());
    /// ```
    pub fn open(root: P) -> Result<Storage<P>> {
        let metadata = try!(fs::metadata(root.as_ref()));
        if !metadata.is_dir() {
            Err(Error::NotADirectory(root.as_ref().as_os_str().to_os_string()))
        } else {
            Ok(Storage { root: root })
        }
    }
}

impl<P: AsRef<Path>> storage::Storage for Storage<P> {
    /// Stores a message on the filesystem.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sbd::storage::{FilesystemStorage, Storage};
    /// use sbd::mo::Message;
    /// let message: Message = Default::default();
    /// let storage = FilesystemStorage::open("/var/iridium").unwrap();
    /// storage.store(&message);
    /// ```
    fn store(&mut self, message: &Message) -> Result<()> {
        let mut path_buf = self.root.as_ref().to_path_buf();
        path_buf.push(message.imei());
        path_buf.push(message.time_of_session().format("%Y").to_string());
        path_buf.push(message.time_of_session().format("%m").to_string());
        try!(fs::create_dir_all(&path_buf));
        path_buf.push(message.time_of_session()
            .format(&format!("%y%m%d_%H%M%S.{}", SBD_EXTENSION))
            .to_string());
        let mut file = try!(fs::File::create(&path_buf));
        try!(message.write_to(&mut file));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    extern crate tempdir;
    use self::tempdir::TempDir;

    use super::*;

    use mo::Message;
    use storage::Storage as StorageTrait;

    #[test]
    fn filesystem_storage_open() {
        Storage::open(TempDir::new("").unwrap().path()).unwrap();
    }

    #[test]
    fn filesystem_storage_dne() {
        assert!(Storage::open("not/a/real/directory").is_err());
    }

    #[test]
    fn filesystem_storage_is_file() {
        assert!(Storage::open("data/0-mo.sbd").is_err());
    }

    #[test]
    fn store_message() {
        let tempdir = TempDir::new("").unwrap();
        let mut storage = Storage::open(tempdir.path()).unwrap();
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        storage.store(&message).unwrap();
        let mut message_path = PathBuf::from(tempdir.path());
        message_path.push("300234063904190");
        message_path.push("2015");
        message_path.push("07");
        message_path.push("150709_181508.sbd");
        Message::from_path(message_path).unwrap();
    }
}
