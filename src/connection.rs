use crate::{
    http::Header,
    request::{Method, Request},
    response::{Response, StatusCode},
};
use anyhow::Result;
use std::io::{prelude::*, BufReader};
use std::net::{Shutdown, TcpStream};

pub trait Shutdownable {
    fn shutdown(&self, how: Shutdown) -> std::io::Result<()>;
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Shutdownable for TcpStream {
    fn shutdown(&self, how: Shutdown) -> std::io::Result<()> {
        self.shutdown(how)
    }
}

#[derive(Debug)]
pub struct Connection<T>
where
    T: Read + Write + Shutdownable,
{
    stream: T,
}

impl<T> Connection<T>
where
    T: Read + Write + Shutdownable + std::fmt::Debug,
{
    pub fn new(stream: T) -> Self {
        println!("Accepting new connection: {stream:?}");
        Self { stream }
    }

    pub fn process(&mut self) -> Result<()> {
        let buf_reader = BufReader::new(&mut self.stream);

        let request = Request::decode(buf_reader)?;
        println!("Received: {request:?}");

        let response = match (request.method, request.target.as_str()) {
            (Method::Get, "/") => Response::new(StatusCode::Ok),
            (Method::Get, target) if target.starts_with("/echo/") => {
                let mut response = Response::new(StatusCode::Ok);
                response.add_header(Header::ContentType("text/plain".to_string()));
                // Safety: Have already checked target starts_with
                let body = target.strip_prefix("/echo/").unwrap();
                response.body(body.into());

                response
            }
            _ => Response::new(StatusCode::NotFound),
        };
        println!("Sending: {response:?}");
        self.stream.write_all(&response.encode())?;

        Ok(())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl<T> Drop for Connection<T>
where
    T: Read + Write + Shutdownable,
{
    fn drop(&mut self) {
        println!("Shutting down connection");
        if let Err(error) = self.stream.shutdown(Shutdown::Both) {
            eprintln!("Error shutting down connection: {error}");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockall::*;

    mock! {
        #[derive(Debug)]
        Connection {}
        impl Read for Connection {
            fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize>;
        }
        impl Write for Connection {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
            fn flush(&mut self) -> std::io::Result<()>;
        }
        impl Shutdownable for Connection {
            fn shutdown(&self, _how: Shutdown) -> std::io::Result<()>;
        }
    }

    fn mock(input: &'static [u8], output: &'static [u8]) -> Result<()> {
        let mut mock = MockConnection::new();
        mock.expect_read().once().returning(|buf| {
            buf[..input.len()].copy_from_slice(input);
            Ok(input.len())
        });
        mock.expect_write()
            .with(predicate::eq(output))
            .once()
            .returning(|buf| Ok(buf.len()));
        mock.expect_shutdown().once().returning(|_| Ok(()));

        Connection::new(mock).process()
    }

    #[test]
    fn get_known_request_target_returns_200() -> Result<()> {
        mock(b"GET / HTTP/1.1\r\n\r\n", b"HTTP/1.1 200 OK\r\n\r\n")
    }

    #[test]
    fn getting_invalid_request_target_returns_404() -> Result<()> {
        mock(
            b"GET /not_found HTTP/1.1\r\n\r\n",
            b"HTTP/1.1 404 Not Found\r\n\r\n",
        )
    }

    #[test]
    fn get_echo_returns_200() -> Result<()> {
        mock(
            b"GET /echo/rust HTTP/1.1\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\n\r\nrust",
        )
    }

    #[test]
    fn method_not_supported_is_unimplemented() {
        let input = b"POST / HTTP/1.1\r\n\r\n";

        let mut mock = MockConnection::new();
        mock.expect_read().once().returning(|buf| {
            buf[..input.len()].copy_from_slice(input);
            Ok(input.len())
        });
        mock.expect_shutdown().once().returning(|_| Ok(()));

        let result = Connection::new(mock).process();
        assert!(result.is_err());
    }
}
