//! Receive incoming Iridium messages through their Direct IP service.
//!
//! Iridium DirectIP is a service provided by the Iridium company. New Mobile
//! Originated messages are forwarded from the Iridium servers to a configured
//! IP address. The Iridum service attempts to initate a TCP connection to port
//! 10800 at the specified IP. If the connection is successful, the MO message
//! is transmitted, then the connection is closed.
//!
//! This module provides a `Server` structure, which can be created to run
//! forever and receive those incoming MO messages.

use std::io;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use filesystem::Storage;
use mo::Message;

/// A Iridium DirectIP server.
///
/// The server will listen on a socket address for incoming Iridium SBD Mobile Originated
/// messages. Incoming messages will be stored using `sbd::filesystem::Storage`. Errors are logged
/// using the logging framework.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Server<A: ToSocketAddrs + Sync> {
    addr: A,
    listener: Option<TcpListener>,
    storage: Arc<Storage>,
}

impl<A: ToSocketAddrs + Sync> Server<A> {
    /// Creates a new server that will listen on `addr` and write messages to `root`.
    ///
    /// This method does not actually bind to the socket address or do anything with the root
    /// directory. Use `bind` and `serve_forever` to actually do stuff.
    ///
    /// The `root` parameters is used to create a new `sbd::filesystem::Storage`, and the server
    /// will store all incoming messages under `root`.
    ///
    /// # Examples
    ///
    /// ```
    /// let server = sbd::directip::Server::new("0.0.0.0:10800", "/var/iridium");
    /// ```
    pub fn new<P: AsRef<Path>>(addr: A, root: P) -> Server<A> {
        Server {
            addr: addr,
            listener: None,
            storage: Arc::new(Storage::new(root)),
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
    /// let mut server = sbd::directip::Server::new("0.0.0.0:10800", "/var/iridium");
    /// server.bind().unwrap();
    /// ```
    pub fn bind(&mut self) -> io::Result<()> {
        self.listener = Some(try!(self.create_listener()));
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
    /// let mut server = sbd::directip::Server::new("0.0.0.0:10800", "/var/iridium");
    /// server.bind().unwrap();
    /// server.serve_forever();
    /// ```
    pub fn serve_forever(&mut self) {
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
                    let storage = self.storage.clone();
                    thread::spawn(move || handle_stream(stream, storage));
                }
                Err(err) => {
                    thread::spawn(move || handle_error(err));
                }
            }
        }
    }

    fn create_listener(&self) -> io::Result<TcpListener> {
        TcpListener::bind(&self.addr)
    }
}

/// Handles an incoming DirectIP stream.
fn handle_stream(stream: TcpStream, storage: Arc<Storage>) {
    match stream.peer_addr() {
        Ok(addr) => {
            debug!("Handling TcpStream from {}", addr);
        }
        Err(err) => {
            warn!("Problem when extracting peer address from TcpStream, but we'll press on: {:?}",
                  err);
        }
    }
    let ref message = match Message::read_from(stream) {
        Ok(message) => {
            info!("Recieved message with {} byte payload",
                  message.payload_ref().len());
            message
        }
        Err(err) => {
            error!("Error when reading message: {:?}", err);
            return;
        }
    };
    match storage.store(message) {
        Ok(path) => info!("Stored message to {:?}", path),
        Err(err) => error!("Problem storing message: {:?}", err),
    }
}

/// Handles an error when handling a connection.
fn handle_error(err: io::Error) {
    error!("Error when receiving tcp communication: {:?}", err);
}
