//! A one-request HTTP server for unit tests: serves a canned JSON body and
//! reports the request target the client sent, so each test checks both the
//! URL a method builds and that the response parses.

use crate::client::Client;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver};

/// A pro client pointed at the mock server (with a key, so onchain paths
/// are not redirected to `GeckoTerminal`)
pub(crate) struct Mock {
    pub(crate) client: Client,
    requests: Receiver<(String, String)>,
}

impl Mock {
    /// Serve `body` with status 200 to the next request
    pub(crate) fn new(body: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let body = body.to_string();
        let (tx, requests) = channel();
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).unwrap();
            let mut api_key = String::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" || line.is_empty() {
                    break;
                }
                let lower = line.to_ascii_lowercase();
                if let Some(v) = lower.strip_prefix("x-cg-pro-api-key:") {
                    api_key = v.trim().to_string();
                }
            }
            let target = request_line.split(' ').nth(1).unwrap_or_default();
            tx.send((target.to_string(), api_key)).unwrap();
            let mut stream = reader.into_inner();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\
                 content-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        });
        let client = Client::pro("test-key")
            .unwrap()
            .with_base_url(&format!("http://{addr}/api/v3"));
        Self { client, requests }
    }

    /// The path and query the client requested, below `/api/v3`
    pub(crate) fn request(&self) -> String {
        let (target, api_key) = self.requests.recv().unwrap();
        assert_eq!(api_key, "test-key", "API key header");
        target.strip_prefix("/api/v3").unwrap().to_string()
    }
}
