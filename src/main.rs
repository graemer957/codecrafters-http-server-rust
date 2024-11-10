use anyhow::Result;
use std::{
    io::{prelude::*, BufReader},
    net::TcpListener,
};
use thiserror::Error;

#[derive(Error, Debug)]
enum ServerError {
    #[error("The HTTP request is missing the request line (method, request target and version")]
    MissingRequestLine,
    #[error("Element missing from request line (method, request target or version")]
    InvalidRequestLine,
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection: {stream:?}");
                let buf_reader = BufReader::new(&mut stream);
                let request_line = buf_reader
                    .lines()
                    .next()
                    .ok_or(ServerError::MissingRequestLine)??;
                let mut parts = request_line.split_whitespace();
                let (method, request_target, version) =
                    match (parts.next(), parts.next(), parts.next()) {
                        (Some(method), Some(request_target), Some(version)) => {
                            (method, request_target, version)
                        }
                        _ => Err(ServerError::InvalidRequestLine)?,
                    };
                println!("Method: '{method}' | Path: '{request_target}' | Version: '{version}'");

                match request_target {
                    "/" => stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")?,
                    _ => stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")?,
                };
            }
            Err(e) => {
                eprintln!("error: {e}");
            }
        }
    }

    Ok(())
}
