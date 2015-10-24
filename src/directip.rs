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
use std::thread;

/// A Iridium DirectIP server.
pub struct Server<A: ToSocketAddrs + Sync> {
    addr: A,
}

impl<A: ToSocketAddrs + Sync> Server<A> {
    /// Creates a new server that will bind to the given address.
    pub fn new(addr: A) -> Server<A> {
        Server { addr: addr }
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
                    thread::spawn(move || {
                        handle_stream(stream)
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
fn handle_stream(stream: TcpStream) {

}

/// Handles an error when handling a connection.
fn handle_error(err: io::Error) {

}
