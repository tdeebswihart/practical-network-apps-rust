use std::io::{self, Read, Write};
use std::net::TcpListener;

const PING: &str = "*1\r\n$4\r\nPING\r\n";
const PONG: &str = "+PONG\r\n";

// I'd probably use tokio if I cared here
fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buffer = [0; PING.len()];
        let mut read = 0;
        while read < PING.len() {
            let rd = stream.read(&mut buffer[read..])?;
            if rd == 0 {
                break;
            }
            read += rd;
        }
        assert_eq!(String::from_utf8_lossy(&buffer[..]), PING.to_string());
        stream.write_all(PONG.as_bytes())?;
    }

    Ok(())
}
