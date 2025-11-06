#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use anyhow::Result;
use clap::Parser;
use connection::Connection;
use std::{net::TcpListener, time::Duration};
use threadpool::ThreadPool;

mod connection;
mod http;
mod request;
mod response;
mod threadpool;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    directory: Option<String>,
}

// Only wait a maximum of 5 seconds for data for the client
// This mitegates clients that connect and do nothing, but does nothing for clients that
// drip feel (added into README > TODO)
const RECEIVE_TIMEOUT: u64 = 5;

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(&args);

    let listener = TcpListener::bind("127.0.0.1:4221")?;
    let pool = ThreadPool::new(4);

    loop {
        let (stream, _) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(RECEIVE_TIMEOUT)))?;
        let mut connection = Connection::new(stream, args.directory.clone());
        pool.execute(move || {
            if let Err(err) = connection.process() {
                eprintln!("Connection error: {err}");
            }
        });
    }
}
