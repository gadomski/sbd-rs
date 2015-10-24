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
    listener: Option<TcpListener>,
    storage: Arc<Storage>,
}

impl<A: ToSocketAddrs + Sync> Server<A> {
    /// Creates a new server.
    ///
    /// This function does not bind to the address or do anything with the root directory.
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
    pub fn bind(&mut self) -> io::Result<()> {
        self.listener = Some(try!(self.create_listener()));
        Ok(())
    }

    /// Starts the DirectIP server and serve forever.
    ///
    /// # Panics
    ///
    /// This method panics if it has a problem binding to the tcp socket address. To avoid a panic,
    /// use `Server::bind` before calling `Server::serve_forever`.
    pub fn serve_forever(&mut self) {
        let listener = match self.listener {
            Some(ref listener) => listener,
            None => {
                self.listener = Some(self.create_listener().unwrap());
                self.listener.as_ref().unwrap()
            },
        };
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

    fn create_listener(&self) -> io::Result<TcpListener> {
        TcpListener::bind(&self.addr)
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
