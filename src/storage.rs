//! Manage backend SBD message storage.

use std::fs;
use std::path::{Path, PathBuf};

use {Result, Error};
use mo::Message;

const SBD_EXTENSION: &'static str = ".sbd";

/// Trait for all backend SBD storages.
pub trait Storage {
    /// Stores message in this storage.
    fn store(&mut self, message: &Message) -> Result<()>;
}

/// A structure for managing storing and retriving SBD messages on a filesystem.
///
/// Messages are stored in a directory hierarchy under a single root directory.
/// Message storage and retrieval are managed by a `Storage` object, which is
/// configured for a single root directory.
#[derive(Debug)]
pub struct FilesystemStorage {
    root: PathBuf,
}

impl FilesystemStorage {
    /// Opens a new storage for a given directory.
    ///
    /// If the directory does not exist, returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::storage::FilesystemStorage;
    /// let storage = FilesystemStorage::open("data").unwrap();
    /// assert!(FilesystemStorage::open("not/a/directory").is_err());
    /// ```
    pub fn open<P: AsRef<Path>>(root: P) -> Result<FilesystemStorage> {
        let metadata = try!(fs::metadata(root.as_ref()));
        if !metadata.is_dir() {
            Err(Error::NotADirectory(root.as_ref().as_os_str().to_os_string()))
        } else {
            Ok(FilesystemStorage { root: root.as_ref().to_path_buf() })
        }
    }
}

impl Storage for FilesystemStorage {
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
        let mut path_buf = self.root.clone();
        path_buf.push(message.imei());
        path_buf.push(message.time_of_session().format("%Y").to_string());
        path_buf.push(message.time_of_session().format("%m").to_string());
        try!(fs::create_dir_all(&path_buf));
        path_buf.push(message.time_of_session()
            .format(&format!("%y%m%d_%H%M%S{}", SBD_EXTENSION))
            .to_string());
        let mut file = try!(fs::File::create(&path_buf));
        try!(message.write_to(&mut file));
        Ok(())
    }
}

/// A simple storage backend that saves the messages in memory.
///
/// This shouldn't be used for persistent storage.
#[derive(Debug)]
pub struct MemoryStorage {
    messages: Vec<Message>,
}

impl MemoryStorage {
    /// Creates a new memory storage.
    ///
    /// # Examples
    ///
    /// ```
    /// let storage = sbd::storage::MemoryStorage::new();
    /// ```
    pub fn new() -> MemoryStorage {
        MemoryStorage { messages: Vec::new() }
    }

    /// Returns a reference to the underlying message vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::storage::Storage;
    /// let mut storage = sbd::storage::MemoryStorage::new();
    /// assert!(storage.messages().is_empty());
    /// storage.store(&sbd::mo::Message::from_path("data/0-mo.sbd").unwrap());
    /// assert_eq!(1, storage.messages().len());
    /// ```
    pub fn messages(&self) -> &Vec<Message> {
        &self.messages
    }
}

impl Storage for MemoryStorage {
    fn store(&mut self, message: &Message) -> Result<()> {
        self.messages.push((*message).clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use super::*;

    use std::path::PathBuf;

    use self::tempdir::TempDir;

    use mo::Message;

    #[test]
    fn filesystem_storage_open() {
        FilesystemStorage::open(TempDir::new("").unwrap().path()).unwrap();
    }

    #[test]
    fn filesystem_storage_dne() {
        assert!(FilesystemStorage::open("not/a/real/directory").is_err());
    }

    #[test]
    fn filesystem_storage_is_file() {
        assert!(FilesystemStorage::open("data/0-mo.sbd").is_err());
    }

    #[test]
    fn store_message() {
        let tempdir = TempDir::new("").unwrap();
        let mut storage = FilesystemStorage::open(tempdir.path()).unwrap();
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        storage.store(&message).unwrap();
        let mut message_path = PathBuf::from(tempdir.path());
        message_path.push("300234063904190");
        message_path.push("2015");
        message_path.push("07");
        message_path.push("150709_181508.sbd");
        Message::from_path(message_path).unwrap();
    }

    #[test]
    fn store_memory() {
        let mut storage = MemoryStorage::new();
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        storage.store(&message).unwrap();
        let ref stored_message = storage.messages()[0];
        assert_eq!(&message, stored_message);
    }
}
