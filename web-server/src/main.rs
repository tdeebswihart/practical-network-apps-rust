// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;
extern crate clap;
use clap::{App, Arg};

extern crate dotenv;

use dotenv::dotenv;
use std::env;
use std::io::prelude::*;
use std::net::{ TcpListener, TcpStream };

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}

use errors::*;

quick_main!(run);

fn run() -> Result<()> {
    dotenv().ok();
    let matches = App::new("web-server")
        .version("0.1.0")
        .author("Tim Deeb-Swihart <tim@deebswih.art>")
        .about("Runs a multithreaded server from the rust book")
        .arg(
            Arg::with_name("port")
                .short('p')
                .about("Sets the server port. Defaults to 7878")
                .takes_value(true)
                .default_value("7878"),
        )
        .arg(
            Arg::with_name("host")
                .short('a')
                .about("Sets the host to listen on. Defaults to '127.0.0.1'")
                .takes_value(true)
                .default_value("127.0.0.1"),
        )
        .get_matches();

    let host = matches.value_of("host").ok_or("host must be specified")?;
    let port = matches.value_of("port").ok_or("port must be specified")?;

    let listener = TcpListener::bind(format!("{}:{}", host, port))
        .chain_err(|| "unable to bind port")?;
    println!("listening on {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("connection established");
        if let Err(e) = handle_connection(stream){
            eprintln!("failed to read stream: {}", e);
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buffer =[0;1024];
    stream.read(&mut buffer).chain_err(|| "failed to read stream")?;

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    Ok(())
}
