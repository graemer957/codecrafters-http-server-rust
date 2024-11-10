use anyhow::Result;
use std::{io::Write, net::TcpListener};

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection: {stream:?}");
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")?;
            }
            Err(e) => {
                eprintln!("error: {e}");
            }
        }
    }

    Ok(())
}
