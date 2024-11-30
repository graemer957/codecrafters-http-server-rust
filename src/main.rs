#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use anyhow::Result;
use clap::Parser;
use connection::Connection;
use std::net::TcpListener;
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

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(&args);

    let listener = TcpListener::bind("127.0.0.1:4221")?;
    let pool = ThreadPool::new(4);

    loop {
        let (stream, _) = listener.accept()?;
        let mut connection = Connection::new(stream, args.directory.clone());
        pool.execute(move || {
            let _ = connection.process();
        });
    }
}
