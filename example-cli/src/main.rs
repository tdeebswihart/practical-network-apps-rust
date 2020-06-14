extern crate clap;
use clap::{load_yaml, App};

extern crate dotenv;

use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    let yaml = load_yaml!("cli.yaml");

    let matches = App::from(yaml).get_matches();

    // Clap returns Option<&str> while the env returns Result<String>
    // We need to normalize our types here by changing the clap one to an owned string
    let conf = matches.value_of("config")
        .map(|s| s.to_owned())
        .or(env::var("CONFIG").ok())
        .unwrap_or("~/.config/example-cli/conf.yaml".to_string());
    println!("conf is {:?}", conf);

    match matches.occurrences_of("v") {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        3 | _ => println!("Don't be crazy"),
    }
if let Some(matches) = matches.subcommand_matches("test") {
        if matches.is_present("debug") {
            println!("Printing debug info...");
        } else {
            println!("Printing normally...");
        }
    }
}
