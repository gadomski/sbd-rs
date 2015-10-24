//! Receive incoming Iridium messages through their Direct IP service.
//!
//! Iridium DirectIP is a service provided by the Iridium company. New Mobile Originated messages
//! are forwarded from the Iridium servers to a configured IP address. The Iridum service attempts
//! to initate a TCP connection to port 10800 at the specified IP. IF the connection is successful,
//! the MO message is transmitted, then the connection is closed.
//!
//! This module provides a `Server` structure, which can be created to run forever and receive
//! those incoming MO messages.

use std::io;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use super::filesystem::Storage;
use super::message::Message;

/// A Iridium DirectIP server.
pub struct Server<A: ToSocketAddrs + Sync> {
    addr: A,
    storage: Arc<Storage>,
}

impl<A: ToSocketAddrs + Sync> Server<A> {
    /// Creates a new server that will bind to the given address.
    pub fn new<P: AsRef<Path>>(addr: A, root: P) -> Server<A> {
        Server {
            addr: addr,
            storage: Arc::new(Storage::new(root)),
        }
    }

    /// Starts the DirectIP server and serve forever.
    ///
    /// # Panics
    ///
    /// This method panics, instead of returning errors, because it's never supposed to exit.
    /// Some reasons why it might panic:
    ///
    /// - `TcpListener` can't bind to the server's socket address.
    pub fn serve_forever(&self) {
        let listener = TcpListener::bind(&self.addr).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let storage = self.storage.clone();
                    thread::spawn(move || {
                        handle_stream(stream, storage)
                    });
                }
                Err(err) => {
                    thread::spawn(move || {
                        handle_error(err)
                    });
                }
            }
        }
    }
}

/// Handles an incoming DirectIP stream.
fn handle_stream(stream: TcpStream, storage: Arc<Storage>) {
    let ref message = Message::read_from(stream).unwrap();
    storage.store(message).unwrap();
}

/// Handles an error when handling a connection.
fn handle_error(err: io::Error) {

}
