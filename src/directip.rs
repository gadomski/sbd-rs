//! Receive incoming Iridium messages through their Direct IP service.
//!
//! Iridium DirectIP is a service provided by the Iridium company. New Mobile Originated messages
//! are forwarded from the Iridium servers to a configured IP address. The Iridum service attempts
//! to initate a TCP connection to port 10800 at the specified IP. IF the connection is successful,
//! the MO message is transmitted, then the connection is closed.
//!
//! This module provides a `Server` structure, which can be created to run forever and receive
//! those incoming MO messages.

/// A Iridium DirectIP server.
pub struct Server;

impl Server {
    /// Start the DirectIP server and serve forever.
    pub fn serve_forever() {

    }
}
