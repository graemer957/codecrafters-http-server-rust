use crate::{
    request::{Method, Request},
    response::{Response, StatusCode},
};
use anyhow::Result;
use std::io::{prelude::*, BufReader};
use std::net::{Shutdown, TcpStream};

pub trait Shutdownable {
    fn shutdown(&self, how: Shutdown) -> std::io::Result<()>;
}

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
            _ => Response::new(StatusCode::NotFound),
        };
        println!("Sending: {response:?}");
        self.stream.write_all(&response.encode())?;

        Ok(())
    }
}

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

    #[derive(Debug)]
    struct MockConnection<'a> {
        input: &'a [u8],
        output: Vec<u8>,
    }

    impl<'a> std::io::Read for MockConnection<'a> {
        fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
            let bytes_written = buf.write(self.input)?;
            Ok(bytes_written)
        }
    }

    impl<'a> std::io::Write for MockConnection<'a> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.output = Vec::from(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> Shutdownable for MockConnection<'a> {
        fn shutdown(&self, _how: Shutdown) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn it_works() {
        let input = b"GET / HTTP/1.1\r\n\r\n";
        let output = b"HTTP/1.1 200 OK\r\n\r\n";

        let mock = MockConnection {
            input,
            output: vec![],
        };
        let mut connection = Connection::new(mock);
        let result = connection.process();

        assert!(result.is_ok());
        assert_eq!(connection.stream.output, output);
    }
}
