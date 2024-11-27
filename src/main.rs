#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use anyhow::Result;
use std::net::TcpListener;

mod connection;
mod http;
mod request;
mod response;
mod threadpool;

use connection::Connection;
use threadpool::ThreadPool;

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;
    let pool = ThreadPool::new(4);

    loop {
        let (stream, _) = listener.accept()?;
        let mut connection = Connection::new(stream);
        pool.execute(move || {
            let _ = connection.process();
        });
    }
}
