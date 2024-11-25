use crate::http;
use anyhow::Result;
use std::{collections::HashMap, io::BufRead};
use thiserror::Error;

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub target: String,
    pub headers: HashMap<String, String>,
}

impl Request {
    pub fn decode<T: BufRead>(reader: T) -> Result<Self> {
        let mut lines = reader.lines();
        let Some(Ok(request_line)) = lines.next() else {
            return Err(Error::MissingRequestLine.into());
        };
        let mut parts = request_line.split_whitespace();

        let method = if let Some(method) = parts.next() {
            Method::decode(method)?
        } else {
            return Err(Error::MissingHTTPMethod.into());
        };

        let request_target = parts.next().ok_or(Error::MissingRequestTarget)?;

        if let Some(version) = parts.next() {
            if version != http::VERSION {
                return Err(Error::UnsupportedHTTPVersion.into());
            }
        } else {
            return Err(Error::MissingHTTPVersion.into());
        };

        let mut headers = HashMap::new();
        while let Some(Ok(header)) = lines.next() {
            if header.is_empty() {
                break;
            }

            let mut split = header.split_terminator(':');
            match (split.next(), split.next()) {
                // Technically I think we should return 400 to client if key has any whitespace
                (Some(k), Some(v)) => headers.insert(k.trim().to_lowercase(), v.trim().to_string()),
                _ => return Err(Error::InvalidHeader.into()),
            };
        }

        Ok(Self {
            method,
            target: String::from(request_target),
            headers,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Method {
    Get,
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
}

impl Method {
    pub fn decode(data: &str) -> Result<Self> {
        match data {
            "GET" => Ok(Self::Get),
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
}
