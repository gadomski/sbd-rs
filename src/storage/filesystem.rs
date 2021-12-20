//! Store SBD messages on the filesystem.

use std::{
    ffi::OsString,
    fmt, fs,
    path::{Path, PathBuf},
};

use failure::Error;
use walkdir;

use crate::{mo::Message, storage};

const SBD_EXTENSION: &str = "sbd";

/// A structure for managing storing and retriving SBD messages on a filesystem.
///
/// Messages are stored in a directory hierarchy under a single root directory.
/// Message storage and retrieval are managed by a `Storage` object, which is
/// configured for a single root directory.
#[derive(Clone, Debug)]
pub struct Storage {
    root: PathBuf,
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
#[derive(Debug)]
pub struct StorageIterator {
    iter: walkdir::IntoIter,
}

/// An error returned when trying to create a storage for a non-directoy.
#[derive(Debug, Fail)]
pub struct NotADirectory(OsString);

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
    pub fn open<P: AsRef<Path>>(root: P) -> Result<Storage, ::failure::Error> {
        let metadata = fs::metadata(root.as_ref())?;
        if !metadata.is_dir() {
            Err(NotADirectory(root.as_ref().as_os_str().to_os_string()).into())
        } else {
            Ok(Storage {
                root: root.as_ref().to_path_buf(),
            })
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
        StorageIterator::new(&self.root)
    }
}

impl storage::Storage for Storage {
    fn store(&mut self, message: Message) -> Result<(), ::failure::Error> {
        let mut path_buf = self.root.clone();
        path_buf.push(message.imei());
        path_buf.push(message.time_of_session().format("%Y").to_string());
        path_buf.push(message.time_of_session().format("%m").to_string());
        fs::create_dir_all(&path_buf)?;
        path_buf.push(
            message
                .time_of_session()
                .format(&format!("%y%m%d_%H%M%S.{}", SBD_EXTENSION))
                .to_string(),
        );
        let mut file = fs::File::create(&path_buf)?;
        message.write_to(&mut file)?;
        Ok(())
    }

    fn messages(&self) -> Result<Vec<Message>, ::failure::Error> {
        self.iter().collect()
    }

    fn messages_from_imei(&self, imei: &str) -> Result<Vec<Message>, ::failure::Error> {
        let mut path = self.root.clone();
        path.push(imei);
        StorageIterator::new(&path).collect()
    }
}

impl StorageIterator {
    fn new(root: &Path) -> StorageIterator {
        StorageIterator {
            iter: walkdir::WalkDir::new(root).into_iter(),
        }
    }
}

impl Iterator for StorageIterator {
    type Item = Result<Message, ::failure::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .by_ref()
            .find(|r| {
                r.as_ref()
                    .map(|d| d.path().extension().map_or(false, |e| e == SBD_EXTENSION))
                    .unwrap_or(false)
            })
            .map(|r| {
                r.map_err(Error::from)
                    .and_then(|d| Message::from_path(d.path()))
            })
    }
}

impl fmt::Display for NotADirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "not a directory: {}", self.0.to_string_lossy())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempdir::TempDir;

    use super::*;
    use crate::{mo::Message, storage::Storage as StorageTrait};

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
        storage.store(message).unwrap();
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
        assert_eq!(0, storage.iter().count());
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        storage.store(message).unwrap();
        assert_eq!(1, storage.iter().count());
    }

    #[test]
    fn messages_from_imei() {
        let tempdir = TempDir::new("").unwrap();
        let mut storage = Storage::open(tempdir.path()).unwrap();
        let message = Message::from_path("data/0-mo.sbd").unwrap();
        storage.store(message.clone()).unwrap();
        let messages = storage.messages_from_imei("300234063904190").unwrap();
        assert_eq!(vec![message], messages);
        let messages = storage.messages_from_imei("300234063904191").unwrap();
        assert!(messages.is_empty());
    }
}
