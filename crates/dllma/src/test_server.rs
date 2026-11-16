//! A minimal local HTTP server for unit tests
//!
//! Answers every request with one canned JSON body and records each
//! request's path (with query string), so a test can check both the route a
//! method calls and how it reads the response.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// A server answering with `status` and `body`
pub(crate) struct TestServer {
    /// Base URL, e.g. `http://127.0.0.1:12345`
    pub url: String,
    paths: Receiver<String>,
}

impl TestServer {
    /// Serve `body` with status 200
    pub fn json(body: &str) -> Self {
        Self::with_status(200, body)
    }

    /// Serve `body` with `status`
    pub fn with_status(status: u16, body: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let url = format!("http://{}", listener.local_addr().expect("local addr"));
        let body = body.to_owned();
        let (tx, paths) = channel();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { return };
                let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() {
                    continue;
                }
                // Skip the headers; GET requests have no body
                let mut line = String::new();
                while reader.read_line(&mut line).is_ok_and(|n| n > 2) {
                    line.clear();
                }
                let path = request_line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .to_owned();
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                if tx.send(path).is_err() {
                    return;
                }
            }
        });
        Self { url, paths }
    }

    /// The path and query of the next request the server received
    pub fn path(&self) -> String {
        self.paths
            .recv_timeout(Duration::from_secs(5))
            .expect("the server got a request")
    }
}
