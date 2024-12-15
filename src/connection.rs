use crate::{
    http::{Header, SUPPORTED_ENCODINGS},
    request::{Method, Request},
    response::{Response, StatusCode},
};
use anyhow::Result;
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{Shutdown, TcpStream},
    path::PathBuf,
};

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
    directory: Option<String>,
}

impl<T> Connection<T>
where
    T: Read + Write + Shutdownable + std::fmt::Debug,
{
    pub fn new(stream: T, directory: Option<String>) -> Self {
        println!("Accepting new connection: {stream:?}");
        Self { stream, directory }
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
                if let Some(encoding) = request.headers.get("accept-encoding") {
                    // Presumably a real server would need to think about casing (or follow
                    // the RFC assuming it was mentioned in there)
                    if SUPPORTED_ENCODINGS.contains(&&encoding[..]) {
                        response.add_header(Header::ContentEncoding("gzip".to_string()));
                    }
                }
                // Safety: Have already checked target starts_with
                let body = target.strip_prefix("/echo/").unwrap();
                response.body(body.into());

                response
            }
            (Method::Get, "/user-agent") => request.headers.get("user-agent").map_or_else(
                || Response::new(StatusCode::BadRequest),
                |user_agent| {
                    let mut response = Response::new(StatusCode::Ok);
                    response.add_header(Header::ContentType("text/plain".to_string()));
                    response.body(user_agent.to_owned().into());

                    response
                },
            ),
            (Method::Get, target) if target.starts_with("/files/") => {
                let mut path_buf = PathBuf::new();
                if let Some(path) = &self.directory {
                    path_buf.push(path);
                };
                // Safety: Have already checked target starts_with
                let filename = target.strip_prefix("/files/").unwrap();
                path_buf.push(filename);
                fs::read(path_buf).map_or_else(
                    |_| Response::new(StatusCode::NotFound),
                    |file_contents| {
                        let mut response = Response::new(StatusCode::Ok);
                        response.add_header(Header::ContentType(
                            "application/octet-stream".to_string(),
                        ));
                        response.body(file_contents);

                        response
                    },
                )
            }
            (Method::Post, target) if target.starts_with("/files") => {
                let mut path_buf = PathBuf::new();
                if let Some(path) = &self.directory {
                    path_buf.push(path);
                };
                // Safety: Have already checked target starts_with
                let filename = target.strip_prefix("/files/").unwrap();
                path_buf.push(filename);
                let _ = fs::write(path_buf, request.body.unwrap());
                Response::new(StatusCode::Created)
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

        Connection::new(mock, None).process()
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
        let input = b"BOOM / HTTP/1.1\r\n\r\n";

        let mut mock = MockConnection::new();
        mock.expect_read().once().returning(|buf| {
            buf[..input.len()].copy_from_slice(input);
            Ok(input.len())
        });
        mock.expect_shutdown().once().returning(|_| Ok(()));

        let result = Connection::new(mock, None).process();
        assert!(result.is_err());
    }

    #[test]
    fn get_user_agent_returns_200() -> Result<()> {
        mock(
            b"GET /user-agent HTTP/1.1\r\nUser-Agent: rust\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\n\r\nrust",
        )
    }

    #[test]
    fn get_user_agent_returns_400() -> Result<()> {
        mock(
            b"GET /user-agent HTTP/1.1\r\n\r\n",
            b"HTTP/1.1 400 Bad Request\r\n\r\n",
        )
    }

    #[test]
    fn get_missing_file_404() -> Result<()> {
        // A lot of effort to cover the case where we need to fill two buffers!
        // Argubly should be in own test, but given the experimental nature
        // of this project ;-)
        let input_1 = b"GET /files/random12345 HTTP/1.1\r";
        let input_2 = b"\n\r\n";
        let output: &[u8] = b"HTTP/1.1 404 Not Found\r\n\r\n";

        let mut mock = MockConnection::new();
        mock.expect_read().once().returning(|buf| {
            buf[..input_1.len()].copy_from_slice(input_1);
            Ok(input_1.len())
        });
        mock.expect_read().once().returning(|buf| {
            buf[..input_2.len()].copy_from_slice(input_2);
            Ok(input_2.len())
        });
        mock.expect_write()
            .with(predicate::eq(output))
            .once()
            .returning(|buf| Ok(buf.len()));
        mock.expect_shutdown().once().returning(|_| Ok(()));

        Connection::new(mock, None).process()
    }

    #[test]
    fn get_valid_file_200() -> Result<()> {
        mock(
            b"GET /files/.gitattributes HTTP/1.1\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 12\r\n\r\n* text=auto\n",
        )
    }

    #[test]
    fn post_file_201() -> Result<()> {
        mock(
            b"POST /files/junk HTTP/1.1\r\nContent-Type: application/octet-stream\r\nContent-Length: 12\r\n\r\nRust",
            b"HTTP/1.1 201 Created\r\n\r\n",
        )
    }

    #[test]
    fn echo_with_gzip() -> Result<()> {
        mock(
            b"GET /echo/rust HTTP/1.1\r\nAccept-Encoding: gzip\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Type: text/plain\r\nContent-Length: 4\r\n\r\nrust",
        )
    }

    #[test]
    fn echo_with_unsupported_encoding() -> Result<()> {
        mock(
            b"GET /echo/rust HTTP/1.1\r\nAccept-Encoding: br\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\n\r\nrust",
        )
    }
}
