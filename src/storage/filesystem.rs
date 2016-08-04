//! Store SBD messages on the filesystem.

use std::fs;
use std::path::{Path, PathBuf};

use walkdir;

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
pub struct Storage {
    root: PathBuf,
}

impl Storage {
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
    pub fn open<P: AsRef<Path>>(root: P) -> Result<Storage> {
        let metadata = try!(fs::metadata(root.as_ref()));
        if !metadata.is_dir() {
            Err(Error::NotADirectory(root.as_ref().as_os_str().to_os_string()))
        } else {
            Ok(Storage { root: root.as_ref().to_path_buf() })
        }
    }

    /// Returns a `StorageIterator` over the messages in this storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use sbd::storage::FilesystemStorage;
    /// for message in FilesystemStorage::open("data").unwrap().iter() {
    ///     println!("{:?}", message);
    /// }
    /// ```
    pub fn iter(&self) -> StorageIterator {
        StorageIterator::new(&self)
    }
}

impl storage::Storage for Storage {
    /// Stores a message on the filesystem.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sbd::storage::{FilesystemStorage, Storage};
    /// use sbd::mo::Message;
    /// let message: Message = Default::default();
    /// let mut storage = FilesystemStorage::open("/var/iridium").unwrap();
    /// storage.store(&message);
    /// ```
    fn store(&mut self, message: &Message) -> Result<()> {
        let mut path_buf = self.root.clone();
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

/// An iterator over the messages in a `Storage`.
///
/// For now, this iterator will just return all messages with an `sbd` extension under the root of
/// the storage tree. We could try to get smarter and mirror the pattern logic for saving, but for
/// now that's more work and complexity than we need.
///
/// # Errors
///
/// This iterator's `Item` is a `sbd::Result<Message>`, because a file with an `sbd` extension
/// might not convert to a message successfully.
#[allow(missing_debug_implementations)]
pub struct StorageIterator {
    iter: walkdir::Iter,
}

impl StorageIterator {
    fn new(storage: &Storage) -> StorageIterator {
        StorageIterator { iter: walkdir::WalkDir::new(&storage.root).into_iter() }
    }
}

impl Iterator for StorageIterator {
    type Item = Result<Message>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .by_ref()
            .skip_while(|r| {
                r.as_ref()
                    .map(|d| d.path().extension().map(|e| e != SBD_EXTENSION).unwrap_or(true))
                    .unwrap_or(true)
            })
            .next()
            .map(|r| r.map_err(|e| Error::from(e)).and_then(|d| Message::from_path(d.path())))
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
    fn open() {
        Storage::open(TempDir::new("").unwrap().path()).unwrap();
    }

    #[test]
    fn no_directory() {
        assert!(Storage::open("not/a/real/directory").is_err());
    }

    #[test]
    fn file_is_error() {
        assert!(Storage::open("data/0-mo.sbd").is_err());
    }

    #[test]
    fn store() {
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

    #[test]
    fn iter() {
        let tempdir = TempDir::new("").unwrap();
        let mut storage = Storage::open(tempdir.path()).unwrap();
        assert_eq!(0, storage.iter().collect::<Vec<_>>().len());
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        storage.store(&message).unwrap();
        assert_eq!(1, storage.iter().collect::<Vec<_>>().len());
    }
}
