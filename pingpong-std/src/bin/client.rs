use std::io::{self, Read, Write};
use std::net::TcpStream;

const PING: &str = "*1\r\n$4\r\nPING\r\n";
const PONG: &str = "+PONG\r\n";

fn main() -> io::Result<()> {
    let mut socket = TcpStream::connect("127.0.0.1:6379")?;
    socket.write_all(PING.as_bytes())?;
    socket.flush()?;
    let mut buffer = [0; PING.len()];
    let mut read = 0;
    while read < PONG.len() {
        let rd = socket.read(&mut buffer[read..])?;
        if rd == 0 {
            break;
        }
        read += rd;
    }

    assert_eq!(String::from_utf8_lossy(&buffer[..read]), PONG.to_string());

    Ok(())
}
