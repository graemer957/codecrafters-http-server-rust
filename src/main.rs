#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use anyhow::Result;
use std::net::TcpListener;

mod connection;
mod http;
mod request;
mod response;

use connection::Connection;

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;

    loop {
        let (stream, _) = listener.accept()?;
        let mut connection = Connection::new(stream);
        connection.process()?;
    }
}
