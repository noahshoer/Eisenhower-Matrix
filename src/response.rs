use std::{io::Write, net::TcpStream};

use crate::status::StatusCode;

struct Response {
    status: StatusCode,
    body: String,
}

impl Response {
    fn new(status: StatusCode, body: impl Into<String>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }

    fn write_to_stream(&self, mut stream: TcpStream) {
        if let Err(e) = stream.write_all(self.to_string().as_bytes()) {
            eprintln!("Failed to write response to stream: {}", e);
        }
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_line = self.status.as_str();
        let length = self.body.len();
        write!(
            f,
            "HTTP/1.1 {status_line}\r\nContent-Length: {length}\r\n\r\n{}",
            self.body
        )
    }
}

pub fn send_response(stream: TcpStream, status: StatusCode, contents: impl Into<String>) {
    let response = Response::new(status, contents);
    response.write_to_stream(stream);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_display() {
        let response = Response::new(StatusCode::Ok, "Test body");
        let response_str = format!("{}", response);
        assert!(response_str.contains("HTTP/1.1 200 OK"));
        assert!(response_str.contains("Content-Length: 9"));
        assert!(response_str.ends_with("Test body"));
    }
}