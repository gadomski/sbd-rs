//! Manage SBD messages stored on the fileystem.
//!
//! Messages are stored in a directory hierarchy under a single root directory.
//! Message storage and retrieval are managed by a `Storage` object, which is
//! configured for a single root directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::result;

use glob::{glob, Paths, PatternError};

use super::{Message, Result};

const SBD_EXTENSION: &'static str = ".sbd";

/// A structure for managing storing and retriving SBD messages on a filesystem.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Storage {
    root: PathBuf,
}

/// An interator over the messages in a `Storage`.
#[allow(missing_copy_implementations, missing_debug_implementations)]
pub struct StorageIterator {
    paths: Option<Paths>,
}

/// The object yielded by a `StorageIterator`.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct StorageEntry {
    /// The Iridium Message contained in the file.
    pub message: Message,
    /// The path of message on the filesystem.
    pub path_buf: PathBuf,
}

impl Iterator for StorageIterator {
    type Item = StorageEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut paths = match self.paths {
            Some(ref mut paths) => paths,
            None => return None,
        };
        loop {
            let glob_result = match paths.next() {
                Some(result) => result,
                None => return None,
            };
            let path_buf = match glob_result {
                Ok(path_buf) => path_buf,
                Err(_) => continue,
            };
            match Message::from_path(&path_buf) {
                Ok(message) => return Some(StorageEntry {
                    message: message,
                    path_buf: path_buf,
                }),
                Err(_) => continue,
            }
        }
    }
}

impl StorageIterator {
    fn new(storage: &Storage) -> StorageIterator {
        let paths = match storage.glob() {
            Ok(paths) => Some(paths),
            Err(_) => None,
        };
        StorageIterator { paths: paths }
    }
}

impl<'a> IntoIterator for &'a Storage {
    type Item = StorageEntry;
    type IntoIter = StorageIterator;

    fn into_iter(self) -> Self::IntoIter {
        StorageIterator::new(self)
    }
}

impl Storage {
    /// Creates a new storage manager for a given root directory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sbd::filesystem::Storage;
    /// let storage = Storage::new("/var/iridium");
    /// ```
    pub fn new<P: AsRef<Path>>(root: P) -> Storage {
        Storage { root: root.as_ref().to_path_buf() }
    }

    /// Stores a message on the filesystem.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sbd::filesystem::Storage;
    /// use sbd::Message;
    /// let message: Message = Default::default();
    /// let storage = Storage::new("/var/iridium");
    /// storage.store(&message);
    /// ```
    pub fn store(&self, message: &Message) -> Result<PathBuf> {
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
        Ok(path_buf)
    }

    fn glob(&self) -> result::Result<Paths, PatternError> {
        let mut path_buf = self.root.clone();
        path_buf.push("**");
        path_buf.push(format!("*{}", SBD_EXTENSION));
        glob(&path_buf.to_str().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempdir::TempDir;

    use super::super::Message;

    #[test]
    fn new_storage() {
        Storage::new(TempDir::new("").unwrap().path());
    }

    #[test]
    fn store_message() {
        let storage = Storage::new(TempDir::new("").unwrap().path());
        let message: Message = Default::default();
        let pathbuf = storage.store(&message).unwrap();
        Message::from_path(pathbuf).unwrap();
    }

    #[test]
    fn iterate_over_messages() {
        let storage = Storage::new(TempDir::new("").unwrap().path());
        let message: Message = Default::default();
        storage.store(&message).unwrap();
        let messages = storage.into_iter().collect::<Vec<StorageEntry>>();
        assert_eq!(1, messages.len());
    }
}
