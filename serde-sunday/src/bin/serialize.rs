#[macro_use]
extern crate structopt;
use structopt::StructOpt;

#[macro_use]
extern crate anyhow;

use anyhow::{Context, Error, Result};
use human_panic::setup_panic;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Cursor};
use std::path::{Path, PathBuf};

use serde_sunday::Move;

#[derive(Debug, StructOpt)]
#[structopt(name = "serialize", about, author)]
struct Opt {
    #[structopt(subcommand)]
    exercise: Option<Exercise>,
}

#[derive(Debug, StructOpt)]
enum Exercise {
    JSON(JSONOpts),
    RON,
    BSON(BSONOpts),
}

#[derive(Debug, StructOpt)]
struct JSONOpts {
    // Buffer file to serialize to and from
    #[structopt(parse(from_os_str))]
    buffer: PathBuf,
}

#[derive(Debug, StructOpt)]
struct BSONOpts {
    /// Buffer file.
    #[structopt(parse(from_os_str))]
    buffer: PathBuf,
}

fn main() -> Result<()> {
    setup_panic!();

    let opts = Opt::from_args();
    match opts.exercise {
        Some(Exercise::JSON(jopts)) => do_json(jopts),
        Some(Exercise::RON) => do_ron(),
        Some(Exercise::BSON(bopts)) => do_bson(bopts),
        None => {
            eprintln!("must specify an exercise!");
            std::process::exit(1);
        }
    }
}

fn do_json(opts: JSONOpts) -> Result<()> {
    let mut buf = BufWriter::new(File::create(&opts.buffer).context("failed to open file")?);
    let mv = Move::random();
    println!("before: {:?}", &mv);
    serde_json::to_writer(&mut buf, &mv).context("failed to serialize json")?;
    buf.flush()?;

    let bf = BufReader::new(File::open(&opts.buffer)?);
    let nmv: Move = serde_json::from_reader(bf).context("failed to deserialize json")?;
    println!("after : {:?}", &nmv);

    Ok(())
}

fn do_ron() -> Result<()> {
    let mv = Move::random();
    println!("before: {:?}", &mv);
    let mut buf: Vec<u8> = Vec::new();
    ron::ser::to_writer(&mut buf, &mv).context("failed to serialize to ron")?;

    let serialized = std::str::from_utf8(&buf).context("failed to convert buffer to utf8")?;
    println!("serialized : {:?}", &serialized);
    Ok(())
}

fn do_bson(opts: BSONOpts) -> Result<()> {
    let n = 1000;
    let mut file_wr = BufWriter::new(File::create(&opts.buffer).context("failed to open file")?);
    let mut file_rd = File::open(&opts.buffer)?;
    bson_write(n, &mut file_wr)?;
    file_wr.flush()?;
    bson_read(&mut file_rd)?;
    println!("bson_file passed");

    let mut vec_wr: Vec<u8> = Vec::new();
    bson_write(n, &mut vec_wr)?;
    let mut vec_rd = Cursor::new(&vec_wr);
    bson_read(&mut vec_rd)?;
    println!("bson_vec passed");
    Ok(())
}

fn bson_write<W: Write + ?Sized>(n: u32, wr: &mut W) -> Result<()> {
    // Generate our random moves.
    for _ in 0..n {
        let mv = Move::random();
        let bmv = bson::to_bson(&mv).context("failed to serialize to bson")?;
        let doc = bmv.as_document().unwrap();
        doc.to_writer(wr).context("failed to write document")?;
    }
    Ok(())
}

fn bson_read<R: Read + ?Sized>(rd: &mut R) -> Result<()> {
    // deserialization time!
    let mut count = 0;
    loop {
        match bson::Document::from_reader(rd) {
            Ok(doc) => {
                let _mv: Move =
                    bson::from_bson(bson::Bson::Document(doc)).context("failed to deserialize")?;
                count += 1;
            }
            Err(bson::de::Error::IoError(ioerr)) => {
                if ioerr.kind() == std::io::ErrorKind::UnexpectedEof {
                    if count == 1000 {
                        return Ok(());
                    } else {
                        return Err(Error::new(ioerr))
                            .with_context(|| format!("io error after {} moves", count));
                    }
                }
            }
            Err(e) => {
                return Err(Error::new(e));
            }
        }
    }
}
