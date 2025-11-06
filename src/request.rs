use crate::http;
use anyhow::Result;
use std::{
    collections::HashMap,
    io::{BufRead, ErrorKind},
};
use thiserror::Error;

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub target: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl Request {
    const BUFFER_SIZE: usize = 32;

    pub fn decode<T: BufRead>(mut reader: T) -> Result<Self> {
        let mut bytes_received = Vec::<u8>::new();

        loop {
            println!(
                "attempting to read {} bytes from `reader`",
                Self::BUFFER_SIZE
            );
            let mut buffer = [0; Self::BUFFER_SIZE];

            match reader.read(&mut buffer) {
                Ok(0) => {
                    println!("read 0 bytes (end of connection?)");
                    break;
                }
                Ok(read) => {
                    println!("read {read} bytes");
                    bytes_received.extend_from_slice(&buffer[..read]);

                    if read < buffer.len() {
                        println!("did not fill buffer last loop, exiting...");
                        break;
                    }
                }
                Err(err)
                    if err.kind() == ErrorKind::TimedOut || err.kind() == ErrorKind::WouldBlock =>
                {
                    return Err(Error::RequestTimeout.into());
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        }

        let mut bytes_received = bytes_received.as_slice();
        let request_line =
            if let Some(cr_index) = bytes_received.windows(2).position(|x| x == b"\r\n") {
                let result = &bytes_received[..cr_index];
                bytes_received = &bytes_received[cr_index + 2..];
                result
            } else {
                return Err(Error::MissingRequestLine.into());
            };

        let mut parts = request_line.split(|x| x == &b' ');

        let method = match parts.next() {
            Some(method) if !method.is_empty() => Method::decode(method)?,
            _ => return Err(Error::MissingHTTPMethod.into()),
        };

        let request_target = parts.next().ok_or(Error::MissingRequestTarget)?;

        if let Some(version) = parts.next() {
            if version != http::VERSION {
                return Err(Error::UnsupportedHTTPVersion.into());
            }
        } else {
            return Err(Error::MissingHTTPVersion.into());
        }

        let headers_buf: &[u8] = bytes_received
            .windows(4)
            .position(|x| x == b"\r\n\r\n")
            .map_or(&[], |crcr_index| {
                let result = &bytes_received[..crcr_index];
                bytes_received = &bytes_received[crcr_index + 4..];
                result
            });

        let mut headers = HashMap::new();
        let mut lines = headers_buf.lines();
        while let Some(Ok(header)) = lines.next() {
            let mut split = header.splitn(2, ':');
            match (split.next(), split.next()) {
                // Technically I think we should return 400 to client if key has any whitespace
                (Some(k), Some(v)) => headers.insert(k.trim().to_lowercase(), v.trim().to_string()),
                _ => return Err(Error::InvalidHeader.into()),
            };
        }

        let body = if bytes_received.is_empty() {
            None
        } else {
            Some(bytes_received.to_vec())
        };

        Ok(Self {
            method,
            target: String::from_utf8(request_target.to_vec())?,
            headers,
            body,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("The HTTP request is missing the request line (method, request target and version")]
    MissingRequestLine,

    #[error("Unable to ascertain HTTP method")]
    MissingHTTPMethod,

    #[error("Unable to extract request target")]
    MissingRequestTarget,

    #[error("Unable to ascertain HTTP version")]
    MissingHTTPVersion,

    #[error("Unsupported HTTP version")]
    UnsupportedHTTPVersion,

    #[error("Unsupported HTTP Method")]
    UnsupportedMethod,

    #[error("Invalid HTTP header")]
    InvalidHeader,

    #[error("Request timeout: did not send data in timely fashion")]
    RequestTimeout,
}

impl Method {
    pub fn decode(data: &[u8]) -> Result<Self> {
        match data {
            b"GET" => Ok(Self::Get),
            b"POST" => Ok(Self::Post),
            _ => Err(Error::UnsupportedMethod.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let input = b"GET / HTTP/1.1\r\nUser-Agent: Rust\r\n\r\n";
        let result = Request::decode(&input[..]).unwrap();

        assert_eq!(result.method, Method::Get);
        assert_eq!(result.target, String::from("/"));
        assert_eq!(result.headers.get("user-agent"), Some(&"Rust".to_string()));

        Ok(())
    }

    #[test]
    fn empty_request() {
        let input = b"";
        let result = Request::decode(&input[..]);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().downcast::<Error>().unwrap(),
            Error::MissingRequestLine
        );
    }

    #[test]
    fn missing_method() {
        let input = b"\r\n";
        let result = Request::decode(&input[..]);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().downcast::<Error>().unwrap(),
            Error::MissingHTTPMethod
        );
    }

    #[test]
    fn missing_target() {
        let input = b"GET\r\n";
        let result = Request::decode(&input[..]);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().downcast::<Error>().unwrap(),
            Error::MissingRequestTarget
        );
    }

    #[test]
    fn missing_version() {
        let input = b"GET /\r\n";
        let result = Request::decode(&input[..]);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().downcast::<Error>().unwrap(),
            Error::MissingHTTPVersion
        );
    }

    #[test]
    fn invalid_version() {
        let input = b"GET / HTTP/1.0\r\n";
        let result = Request::decode(&input[..]);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().downcast::<Error>().unwrap(),
            Error::UnsupportedHTTPVersion
        );
    }

    #[test]
    fn invalid_method() {
        let input = b"DANCE / HTTP/1.1\r\n";
        let result = Request::decode(&input[..]);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().downcast::<Error>().unwrap(),
            Error::UnsupportedMethod
        );
    }

    #[test]
    fn invalid_header() {
        let input = b"GET / HTTP/1.1\r\nBad Header\r\n\r\n";
        let result = Request::decode(&input[..]);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().downcast::<Error>().unwrap(),
            Error::InvalidHeader
        );
    }

    #[test]
    fn header_value_with_colon() -> Result<()> {
        let input = b"GET / HTTP/1.1\r\nHost: localhost:7878\r\n\r\n";
        let result = Request::decode(&input[..])?;

        assert_eq!(
            result.headers.get("host"),
            Some(&"localhost:7878".to_string())
        );

        Ok(())
    }
}
