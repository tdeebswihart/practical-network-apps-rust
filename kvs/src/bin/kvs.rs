extern crate structopt;
use env_logger::Env;
use human_panic::setup_panic;
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;

use kvs::{Error, KvStore, Result};

#[derive(StructOpt, Debug)]
#[structopt(name = "kvs", about, author)]
struct Opts {
    #[structopt(subcommand)]
    commands: Option<Kv>,

    #[structopt(short = "f", long = "file", env = "LOG_FILE")]
    logfile: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
enum Kv {
    #[structopt(name = "set")]
    Set(SetOpts),
    #[structopt(name = "get")]
    Get(GetOpts),
    #[structopt(name = "rm")]
    Rm(RmOpts),
}

#[derive(StructOpt, Debug)]
struct SetOpts {
    #[structopt(name = "KEY")]
    key: String,

    #[structopt(name = "VALUE")]
    value: String,
}

#[derive(StructOpt, Debug)]
struct GetOpts {
    #[structopt(name = "KEY")]
    key: String,
}

#[derive(StructOpt, Debug)]
struct RmOpts {
    #[structopt(name = "KEY")]
    key: String,
}
fn main() -> Result<()> {
    setup_panic!();

    let opts = Opts::from_args();
    let logf = opts.logfile.unwrap_or(PathBuf::from("logd"));
    let mut store = KvStore::open(logf)?;

    match opts.commands {
        Some(Kv::Set(opts)) => {
            store.set(opts.key, opts.value)?;
        }
        Some(Kv::Get(opts)) => {
            match store.get(opts.key)? {
                Some(v) => println!("{}", v),
                None => println!("Key not found"),
            };
        }
        Some(Kv::Rm(opts)) => {
            match store.get(opts.key.clone())? {
                Some(_v) => store.remove(opts.key)?,
                None => println!("Key not found"),
            };
        }
        None => {
            eprintln!("missing command!");
            std::process::exit(1);
        }
    }
    Ok(())
}
