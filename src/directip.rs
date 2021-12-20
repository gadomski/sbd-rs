//! Receive incoming Iridium messages through their Direct IP service.
//!
//! Iridium `DirectIP` is a service provided by the Iridium company. New Mobile
//! Originated messages are forwarded from the Iridium servers to a configured
//! IP address. The Iridum service attempts to initate a TCP connection to port
//! 10800 at the specified IP. If the connection is successful, the MO message
//! is transmitted, then the connection is closed.
//!
//! This module provides a `Server` structure, which can be created to run
//! forever and receive those incoming MO messages.

use crate::mo::Message;
use std::io;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::storage::Storage;

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
            addr: addr,
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
fn handle_stream(stream: TcpStream, storage: Arc<Mutex<Storage>>) {
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
                "Recieved message from IMEI {} with MOMN {} and {} byte payload",
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
        .store(message) {
        Ok(_) => info!("Stored message"),
        Err(err) => error!("Problem storing message: {:?}", err),
    }
}

/// Handles an error when handling a connection.
fn handle_error(err: &io::Error) {
    error!("Error when receiving tcp communication: {:?}", err);
}
