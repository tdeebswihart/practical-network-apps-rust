extern crate structopt;

use structopt::StructOpt;

#[derive(StructOpt,Debug)]
#[structopt(name = "kvs", about, author)]
struct Opts {
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
    match opts.commands {
        Some(Kv::Set(opts)) => {
            eprintln!("unimplemented");
            std::process::exit(1);
        },
        Some(Kv::Get(opts)) => {
            eprintln!("unimplemented");
            std::process::exit(1);
        },
        Some(Kv::Rm(opts))  => {
            eprintln!("unimplemented");
            std::process::exit(1);
        },
        None => {
            eprintln!("missing command!");
            std::process::exit(1);
        }
    }
}
