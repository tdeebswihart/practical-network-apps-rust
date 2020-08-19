extern crate structopt;
use human_panic::setup_panic;
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;

use kvs::{KvStore, Result};

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

fn run(cmd: Kv, logf: impl Into<PathBuf>) -> Result<()> {
    let mut store = KvStore::open(logf)?;

    match cmd {
        Kv::Set(opts) => {
            store.set(opts.key, opts.value)?;
        }
        Kv::Get(opts) => {
            match store.get(opts.key)? {
                Some(v) => print!("{}", v),
                None => print!("Key not found"),
            };
        }
        Kv::Rm(opts) => {
            store.remove(opts.key)?;
        }
    }
    Ok(())
}

fn main() {
    setup_panic!();

    let opts = Opts::from_args();
    let logf = opts
        .logfile
        .unwrap_or(env::current_dir().expect("invalid cwd"));
    if let Some(cmd) = opts.commands {
        if let Err(e) = run(cmd, logf) {
            println!("{}", e);
            std::process::exit(1);
        }
    } else {
        eprintln!("missing command!");
        std::process::exit(1);
    }
}
