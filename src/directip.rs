//! Receive incoming Iridium messages through their Direct IP service.
//!
//! Iridium `DirectIP` is a service provided by the Iridium company. New Mobile
//! Originated messages are forwarded from the Iridium servers to a configured
//! IP address. The Iridium service attempts to initiate a TCP connection to port
//! 10800 at the specified IP. If the connection is successful, the MO message
//! is transmitted, then the connection is closed.
//!
//! This module provides a `Server` structure, which can be created to run
//! forever and receive those incoming MO messages.

use std::{
    io,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::{Arc, Mutex},
    thread,
};

use log::{debug, error, info, warn};

use crate::{mo::Message, storage::Storage};

/// A Iridium `DirectIP` server.
///
/// The server will listen on a socket address for incoming Iridium SBD Mobile Originated
/// messages. Incoming messages will be stored using `sbd::filesystem::Storage`. Errors are logged
/// using the logging framework.
#[derive(Debug)]
pub struct Server<A: ToSocketAddrs + Sync, S: Storage + Sync + Send> {
    addr: A,
    listener: Option<TcpListener>,
    storage: Arc<Mutex<S>>,
}

impl<A, S> Server<A, S>
where
    A: ToSocketAddrs + Sync,
    S: 'static + Storage + Sync + Send,
{
    /// Creates a new server that will listen on `addr` and write messages to `storage`.
    ///
    /// This method does not actually bind to the socket address or do anything with the storage.
    /// Use `bind` and `serve_forever` to actually do stuff.
    ///
    /// The provided storage is expected to be ready to accept new messages.
    ///
    /// # Examples
    ///
    /// ```
    /// let storage = sbd::storage::MemoryStorage::new();
    /// let server = sbd::directip::Server::new("0.0.0.0:10800", storage);
    /// ```
    pub fn new(addr: A, storage: S) -> Server<A, S> {
        Server {
            addr,
            listener: None,
            storage: Arc::new(Mutex::new(storage)),
        }
    }

    /// Binds this server to its tcp socket.
    ///
    /// This is a seperate operation from `serve_forever` so that we can capture any errors
    /// associated with the underlying `TcpListener::bind`.
    ///
    /// # Examples
    ///
    /// ```
    /// let storage = sbd::storage::MemoryStorage::new();
    /// let mut server = sbd::directip::Server::new("0.0.0.0:10800", storage);
    /// server.bind().unwrap();
    /// ```
    pub fn bind(&mut self) -> io::Result<()> {
        self.listener = Some(self.create_listener()?);
        Ok(())
    }

    /// Starts the DirectIP server and serves forever.
    ///
    /// # Panics
    ///
    /// This method panics if it has a problem binding to the tcp socket address. To avoid a panic,
    /// use `Server::bind` before calling `Server::serve_forever`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let storage = sbd::storage::MemoryStorage::new();
    /// let mut server = sbd::directip::Server::new("0.0.0.0:10800", storage);
    /// server.bind().unwrap();
    /// server.serve_forever();
    /// ```
    pub fn serve_forever(mut self) {
        let listener = match self.listener {
            Some(ref listener) => listener,
            None => {
                self.listener = Some(self.create_listener().unwrap());
                self.listener.as_ref().unwrap()
            }
        };
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let storage = Arc::clone(&self.storage);
                    thread::spawn(move || handle_stream(stream, storage));
                }
                Err(err) => {
                    thread::spawn(move || handle_error(&err));
                }
            }
        }
    }

    fn create_listener(&self) -> io::Result<TcpListener> {
        TcpListener::bind(&self.addr)
    }
}

/// Handles an incoming `DirectIP` stream.
fn handle_stream(stream: TcpStream, storage: Arc<Mutex<dyn Storage>>) {
    match stream.peer_addr() {
        Ok(addr) => {
            debug!("Handling TcpStream from {}", addr);
        }
        Err(err) => {
            warn!(
                "Problem when extracting peer address from TcpStream, but we'll press on: {:?}",
                err
            );
        }
    }
    let message = match Message::read_from(stream) {
        Ok(message) => {
            info!(
                "Received message from IMEI {} with MOMN {} and {} byte payload",
                message.imei(),
                message.momsn(),
                message.payload().len(),
            );
            message
        }
        Err(err) => {
            error!("Error when reading message: {:?}", err);
            return;
        }
    };
    match storage
        .lock()
        .expect("unable to lock storage mutex")
        .store(message)
    {
        Ok(_) => info!("Stored message"),
        Err(err) => error!("Problem storing message: {:?}", err),
    }
}

/// Handles an error when handling a connection.
fn handle_error(err: &io::Error) {
    error!("Error when receiving tcp communication: {:?}", err);
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;
    use std::{fs, io::Cursor, path::Path};

    #[test]
    fn test_store_real_message_from_stream_file() {
        // Use a real .mo.sbd file as test input
        let test_file = "data/2-location.mo.sbd";
        if !Path::new(test_file).exists() {
            // Skip test if file does not exist
            eprintln!("Test file {} does not exist, skipping.", test_file);
            return;
        }

        let file_bytes = fs::read(test_file).expect("Failed to read test .mo.sbd file");
        let mut cur = Cursor::new(&file_bytes);
        let parsed_message = match Message::read_from(&mut cur) {
            Ok(msg) => msg,
            Err(e) => panic!("Failed to parse real message: {:?}", e),
        };
        println!(
            "Parsed message from IMEI {} with MOMSN {} and payload length {}",
            parsed_message.imei(),
            parsed_message.momsn(),
            parsed_message.payload().len(),
        );
        assert_eq!(parsed_message.imei(), "301434061799480");
        assert_eq!(parsed_message.momsn(), 7);
        assert_eq!(parsed_message.payload().len(), 46);

        let s = std::str::from_utf8(parsed_message.payload()).expect("payload not UTF-8");
        assert!(s.contains("@gadomski"));

        for ie in parsed_message.information_elements() {
            let mo_location_option = ie.as_mo_location();
            assert!(mo_location_option.is_some());
            let mo_location_result = mo_location_option.unwrap();
            assert!(mo_location_result.is_ok());
            let mo_location = mo_location_result.unwrap();

            assert!(mo_location.north);
            assert!(!mo_location.east);
            assert_eq!(mo_location.cep_km, 2);
        }
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let result = storage.lock().unwrap().store(parsed_message);

        assert!(result.is_ok(), "Real message should be stored successfully");
    }
}
