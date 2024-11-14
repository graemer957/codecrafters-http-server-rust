use crate::http;

#[derive(Debug)]
pub struct Response {
    status_code: StatusCode,
}

impl Response {
    pub const fn new(status_code: StatusCode) -> Self {
        Self { status_code }
    }

    pub fn encode(self) -> Vec<u8> {
        let mut buf = vec![];

        buf.extend(http::VERSION.as_bytes());
        buf.extend(b" ");
        buf.extend(self.status_code.as_bytes());
        buf.extend(b"\r\n");
        buf.extend(b"\r\n");

        buf
    }
}

#[derive(Debug)]
pub enum StatusCode {
    Ok,
    NotFound,
}

impl StatusCode {
    pub const fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ok => b"200 OK",
            Self::NotFound => b"404 Not Found",
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
    fn it_returns_404_not_found() {
        let response = Response::new(StatusCode::NotFound).encode();
        let expected = b"HTTP/1.1 404 Not Found\r\n\r\n";

        assert_eq!(response, expected);
    }
}
