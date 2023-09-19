// Copyright 2019 Contributors to the Parsec project.
// SPDX-License-Identifier: Apache-2.0
//! Service front using TCP sockets
//!
//! Expose Parsec functionality using TCP sockets as an IPC layer.
use super::listener;
use anyhow::Result;
use listener::Listen;
use listener::{Connection,ConnectionMetadata};
use log::error;
use std::io::{Error, ErrorKind};
use std::net::TcpListener;
use std::time::Duration;

static DEFAULT_SOCKET_PORT: u16 = 8000;

/// TCP Socket IPC Manager
#[derive(Debug)]
pub struct TcpSocketListener {
    listener: TcpListener,
    timeout: Duration,
}

impl TcpSocketListener {
    /// Initialise the connection to the TCP socket.
    pub fn new (timeout: Duration, socket_port: u16) -> Result<Self> {
        let address = format!("0.0.0.0:{}", socket_port);
        println!("Listen at {}",address);
        let listener = TcpListener::bind(&address)?;
        Ok(Self { listener, timeout })
    }
}

impl Listen for TcpSocketListener{
    fn set_timeout(&mut self, duration: Duration) {
        self.timeout = duration;
    }

    fn accept(&self) -> Option<Connection> {
        let stream_result = self.listener.accept();
        match stream_result {
            Ok((stream, _)) => {
                if let Err(err) = stream.set_read_timeout(Some(self.timeout)) {
                    format_error!("Failed to set read timeout", err);
                    None
                } else if let Err(err) = stream.set_write_timeout(Some(self.timeout)) {
                    format_error!("Failed to set write timeout", err);
                    None
                } else if let Err(err) = stream.set_nonblocking(false) {
                    format_error!("Failed to set stream as blocking", err);
                    None
                } else {
                    Some(Connection {
                        stream: Box::new(stream),
                        metadata: Some(ConnectionMetadata::UnixPeerCredentials {
                            uid: 1,
                            gid: 1,
                            pid: Some(1),
                            })
                        })
                }
            }
            Err(err) => {
                // Check if the error is because no connections are currently present.
                if err.kind() != ErrorKind::WouldBlock {
                    // Only log the real errors.
                    format_error!("Failed to connect with a TcpStream", err);
                }
                None
            }
        }
    }
}

/// Builder for `TcpSocketListener`
#[derive(Clone, Debug, Default, Copy)]
pub struct TcpSocketListenerBuilder {
    timeout:  Option<Duration>,
    socket_port: Option<u16>
}

impl TcpSocketListenerBuilder {
    /// Create a new TcpSocketListener builder
    pub fn new() -> Self {
        TcpSocketListenerBuilder {
            timeout: None,
             socket_port : Some(DEFAULT_SOCKET_PORT),
        }
    }

    /// Add a timeout on the Tcp Socket used
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// specify the port number
    pub fn with_socket_port(mut self, socket_port: Option<u16>) -> Self {
        self.socket_port = socket_port;
        self
    }

    /// Build the builder into the listener
    pub fn build(self) -> Result<TcpSocketListener> {
        TcpSocketListener::new(
            self.timeout.ok_or_else(|| {
                error!("The listener timeout was not set.");
                Error::new(ErrorKind::InvalidInput, "listener timeout missing")
            })?,
            self.socket_port.
                unwrap_or_else(|| DEFAULT_SOCKET_PORT.into()),
        )
    }
}
