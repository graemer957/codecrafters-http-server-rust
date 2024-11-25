use crate::{http, http::Header};
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct Response {
    status_code: StatusCode,
    // TODO: Sort the headers until it's easier to check responses
    headers: BTreeSet<Header>,
    body: Option<Vec<u8>>,
}

impl Response {
    pub const fn new(status_code: StatusCode) -> Self {
        Self {
            status_code,
            headers: BTreeSet::new(),
            body: None,
        }
    }

    pub fn add_header(&mut self, header: Header) {
        self.headers.insert(header);
    }

    pub fn body(&mut self, body: Vec<u8>) {
        self.add_header(Header::Custom(
            "Content-Length".to_string(),
            body.len().to_string(),
        ));

        self.body = Some(body);
    }

    pub fn encode(self) -> Vec<u8> {
        let mut buf = vec![];

        buf.extend(http::VERSION.as_bytes());
        buf.extend(b" ");
        buf.extend(self.status_code.as_bytes());
        buf.extend(http::CRLF);
        for header in &self.headers {
            buf.extend(header.name().as_bytes());
            buf.extend(b": ");
            buf.extend(header.value().as_bytes());
            buf.extend(http::CRLF);
        }
        buf.extend(http::CRLF);

        if let Some(body) = self.body {
            buf.extend(body);
        }

        buf
    }
}

#[derive(Debug)]
pub enum StatusCode {
    Ok,
    NotFound,
    BadRequest,
}

impl StatusCode {
    pub const fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ok => b"200 OK",
            Self::NotFound => b"404 Not Found",
            Self::BadRequest => b"400 Bad Request",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_returns_200_ok() {
        let response = Response::new(StatusCode::Ok).encode();
        let expected = b"HTTP/1.1 200 OK\r\n\r\n";

        assert_eq!(response, expected);
    }

    #[test]
    fn it_returns_400_bad_request() {
        let response = Response::new(StatusCode::BadRequest).encode();
        let expected = b"HTTP/1.1 400 Bad Request\r\n\r\n";

        assert_eq!(response, expected);
    }

    #[test]
    fn it_returns_404_not_found() {
        let response = Response::new(StatusCode::NotFound).encode();
        let expected = b"HTTP/1.1 404 Not Found\r\n\r\n";

        assert_eq!(response, expected);
    }

    #[test]
    fn it_has_a_custom_header() {
        let mut response = Response::new(StatusCode::Ok);
        response.add_header(Header::Custom("abc".to_string(), "def".to_string()));
        let response = response.encode();
        let expected = b"HTTP/1.1 200 OK\r\nabc: def\r\n\r\n";

        assert_eq!(response, expected);
    }

    #[test]
    fn it_has_a_body() {
        let mut response = Response::new(StatusCode::Ok);
        response.add_header(Header::ContentType("text/plain".to_string()));
        response.body("Hello, world!".into());
        let response = response.encode();

        assert!(response.starts_with(b"HTTP/1.1 200 OK\r\n"));
        assert!(response.ends_with(b"\r\n\r\nHello, world!"));

        // TODO: Not sure if I am missing a more idiomatic way to search for &[u8] in Vec<u8> 🤔
        let contains_subslice = |needle: &[u8]| {
            response
                .windows(needle.len())
                .any(|window| window == needle)
        };
        assert!(contains_subslice(b"Content-Type: text/plain\r\n"));
        assert!(contains_subslice(b"Content-Length: 13\r\n"));
    }
}
