use crate::processes::ThreadPool;
use crate::response::send_response;
use crate::status::StatusCode;

use std::net::{TcpListener, TcpStream};

use std::fs::{self};
use std::io::{BufRead, BufReader};



pub fn run() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let pool = ThreadPool::new(4);

    for stream_result in listener.incoming() {
        let stream = match stream_result {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to establish a connection {e}");
                continue;
            }
        };

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    Ok(())
}

/// Maps a parsed request string (e.g., "GET /") to a filename.
/// Returns Some(filename) if the route is known, or None for 404.
fn resolve_filename(parsed_request: &str) -> Option<&'static str> {
    match parsed_request {
        "GET /" => Some("home.html"),
        // Add more routes here as needed
        _ => None,
    }
}

fn handle_connection(stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        _ => {
            send_response(stream, StatusCode::BadRequest, "");
            return;
        }
    };

    let filename = match parse_request(&request_line).and_then(|parsed| resolve_filename(&parsed)) {
        Some(f) => f,
        None => {
            send_response(stream, StatusCode::NotFound, "");
            return;
        }
    };

    let contents = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(_) => {
            send_response(stream, StatusCode::InternalServerError, "");
            return;
        }
    };

    send_response(stream, StatusCode::Ok, contents);
}

fn parse_request(request: &str) -> Option<String> {
    let mut parts = request.split_whitespace();
    let method = parts.next()?;
    let uri = parts.next()?;

    if let Some(ver) = parts.next() {
        if ver == "HTTP/1.1" {
            return Some([method, uri].join(" "));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::{io::{Read, Write}, thread};

    use super::*;

    fn assert_buffer(buffer: &str, expected: &str) {
        assert!(buffer.contains(expected), "Response was: {}", buffer);
    }

    #[test]
    fn test_resolve_filename() {
        assert_eq!(resolve_filename("GET /"), Some("home.html"));
        assert_eq!(resolve_filename("GET /notfound"), None);
        assert_eq!(resolve_filename("POST /"), None);
    }


    #[test]
    fn test_parse_request_valid() {
        let req = "GET / HTTP/1.1";
        assert_eq!(parse_request(req), Some("GET /".to_owned()));
    }

    #[test]
    fn test_parse_request_invalid_version() {
        let req = "GET / HTTP/2.0";
        assert_eq!(parse_request(req), None);
    }

    #[test]
    fn test_parse_request_incomplete() {
        let req = "GET /";
        assert_eq!(parse_request(req), None);
    }

    #[test]
    fn test_parse_request_empty() {
        let req = "";
        assert_eq!(parse_request(req), None);
    }

    #[test]
    fn test_handle_connection_not_found() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                handle_connection(stream);
            }
        });

        let mut stream = TcpStream::connect(addr).unwrap();
        let request = b"GET /notfound HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(request).unwrap();

        let mut buffer = String::new();
        stream.read_to_string(&mut buffer).unwrap();
        assert_buffer(&buffer, "HTTP/1.1 404 NOT FOUND");
    }

    #[test]
    fn test_handle_connection_bad_request() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                handle_connection(stream);
            }
        });

        let mut stream = TcpStream::connect(addr).unwrap();
        let _ = stream.shutdown(std::net::Shutdown::Write);

        let mut buffer = String::new();
        stream.read_to_string(&mut buffer).unwrap();
        assert_buffer(&buffer, "HTTP/1.1 400 BAD REQUEST");
    }

    #[serial_test::serial]
    #[test]
    fn test_handle_connection_internal_server_error() {
        let temp_dir = std::env::temp_dir();
        std::env::set_current_dir(&temp_dir).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                handle_connection(stream);
            }
        });

        let mut stream = TcpStream::connect(addr).unwrap();
        let request = b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(request).unwrap();

        let mut buffer = String::new();
        stream.read_to_string(&mut buffer).unwrap();
        assert_buffer(&buffer, "HTTP/1.1 500 INTERNAL SERVER ERROR");
    }
}
