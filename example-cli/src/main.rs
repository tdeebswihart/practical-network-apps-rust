extern crate clap;
use clap::{Arg, App};

extern crate dotenv;

use dotenv::dotenv;
use std::env;

// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}

use errors::*;

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

// The above main gives you maximum control over how the error is
// formatted. If you don't care (i.e. you want to display the full
// error during an assert) you can just call the `display_chain` method
// on the error object
#[allow(dead_code)]
fn alternative_main() {
    if let Err(ref e) = run() {
        use error_chain::ChainedError;
        use std::io::Write; // trait which holds `display_chain`
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display_chain()).expect(errmsg);
        ::std::process::exit(1);
    }
}

// Use this macro to auto-generate the main above. You may want to
// set the `RUST_BACKTRACE` env variable to see a backtrace.
// quick_main!(run);

// Most functions will return the `Result` type, imported from the
// `errors` module. It is a typedef of the standard `Result` type
// for which the error type is always our own `Error`.
fn run() -> Result<()> {
    dotenv().ok();
    let matches = App::new("My Super Program")
        .version("0.1.0")
        .author("{{authors}}")
        .about("Does awesome things")
        .arg(Arg::new("config")
             .short('c')
             .long("config")
             .value_name("FILE")
             .about("Sets a custom config file")
             .takes_value(true))
        .arg(Arg::new("INPUT")
             .about("Sets the input file to use")
             .required(true)
             .index(1))
        .arg(Arg::new("v")
             .short('v')
             .multiple(true)
             .about("Sets the level of verbosity"))
        .subcommand(App::new("test")
                    .about("controls testing features")
                    .arg(Arg::new("debug")
                         .short('d')
                         .about("print debug information verbosely")))
        .get_matches();
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
