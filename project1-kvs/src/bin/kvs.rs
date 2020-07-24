extern crate structopt;

use structopt::StructOpt;

#[derive(StructOpt,Debug)]
#[structopt(name = "kvs")]
struct Opts {
    #[structopt(short = "-V")]
    version: bool,

    #[structopt(subcommand)]
    commands: Option<Kv>
}

#[derive(StructOpt,Debug)]
enum Kv {
    #[structopt(name = "set")]
    Set(SetOpts),
    #[structopt(name = "get")]
    Get(GetOpts),
    #[structopt(name = "rm")]
    Rm(RmOpts)
}

#[derive(StructOpt,Debug)]
struct SetOpts {
    #[structopt(name = "KEY")]
    key: String,

    #[structopt(name = "VALUE")]
    value: String
}

#[derive(StructOpt,Debug)]
struct GetOpts {
    #[structopt(name = "KEY")]
    key: String
}

#[derive(StructOpt,Debug)]
struct RmOpts {
    #[structopt(name = "KEY")]
    key: String
}
fn main() {
    let opts = Opts::from_args();
    if opts.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return
    }
    if let Some(command) = opts.commands {
        match command {
            Kv::Set(opts) => {

            },
            Kv::Get(opts) => {
            },
            Kv::Rm(opts)  => {

            }
        }
    } else {
        eprintln!("missing command!")
    }
}
