use bson::de::Error as BsonDeError;
use bson::ser::Error as BsonSerError;
use bson::{Bson, Document};
use log::info;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to create directory {}: {}", path.display(), source))]
    MkDir { source: io::Error, path: PathBuf },
    #[snafu(display("failed to open {}: {}", path.display(), source))]
    Open { source: io::Error, path: PathBuf },
    #[snafu(display("failed to seek: {}", source))]
    LogSeek { source: io::Error },
    #[snafu(display("error deserializing command at offset {}: {}", offset, source))]
    Deser { source: BsonDeError, offset: u64 },
    #[snafu(display("error serializing command {:?}: {}", cmd, source))]
    Ser { source: BsonSerError, cmd: Command },
    #[snafu(display("error writing command to offset {}: {}", offset, source))]
    LogWrite { source: BsonSerError, offset: u64 },
    #[snafu(display("failed to {} at offset {}: {}", action, offset, source))]
    Io {
        action: String,
        source: io::Error,
        offset: u64,
    },
    #[snafu(display("Key {} not found", key))]
    NotFound { key: String },
    #[snafu(display("Expected command {} at offset {}, found {:?}", cmd, offset, found))]
    BadIndex {
        cmd: String,
        offset: u64,
        found: Command,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Log alteration commands.
/// Note: we could probably be faster by using raw bytes and storing
/// value_pos and value_size in the keydir instead of the command position.
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Set { key: String, val: String },
    Rm(String),
}

/// A string to string key-value store
///
/// Key-value pairs are stored in a single log file on disk.
///
/// Example usage:
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::open("/tmp/log.kv")?;
/// store.set("my key".to_owned(), "my value".to_owned())?;
/// let val = store.get("my key".to_owned())?;
/// assert_eq!(val, Some("my value".to_owned()));
///```
pub struct KvStore {
    // TODO: keep multiple log files?
    index: HashMap<String, u64>,
    log_wr: BufWriter<File>,
    log_rd: File,
    pos: u64,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut path = path.into();
        match fs::create_dir_all(&path) {
            Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                    return Err(e).context(MkDir { path: path.clone() });
                }
            }
            Ok(_) => {}
        };
        path.push("log.kv");
        let mut log_rd = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&path)
            .with_context(|| Open { path: path.clone() })?;
        let mut index: HashMap<String, u64> = HashMap::new();
        let mut offset: u64;
        loop {
            offset = log_rd
                .seek(SeekFrom::Current(0))
                .with_context(|| LogSeek {})?;
            match bson::Document::from_reader(&mut log_rd) {
                Ok(doc) => {
                    let cmd: Command =
                        bson::from_bson(Bson::Document(doc)).context(Deser { offset })?;
                    // Apply the command
                    match cmd {
                        Command::Set { key, val } => {
                            println!("setting {}", &key);
                            index.insert(key, offset);
                        }
                        Command::Rm(key) => {
                            index.remove(&key);
                        }
                    }
                }
                Err(bson::de::Error::IoError(ioerr)) => {
                    if ioerr.kind() == std::io::ErrorKind::UnexpectedEof {
                        // this means we're done, unfortunately.
                        break;
                    }
                    return Err(ioerr).with_context(|| Io {
                        action: "deserialize",
                        offset,
                    });
                }
                Err(e) => {
                    return Err(e).with_context(|| Deser { offset });
                }
            }
        }
        let mut log_wr = BufWriter::new(log_rd.try_clone().context(Open { path: path.clone() })?);
        // seek until the end
        let pos = log_wr.seek(SeekFrom::End(0)).with_context(|| LogSeek {})?;
        Ok(KvStore {
            log_rd,
            index,
            log_wr,
            pos,
        })
    }

    /// Retrieve the value stored at the specified key
    ///
    /// Returns `None` if the key does not exist.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if !self.index.contains_key(&key) {
            return Err(Error::NotFound { key });
        }
        // Otherwise seek and get the key
        let offset = self.index.get(&key).unwrap().clone();
        self.log_rd
            .seek(SeekFrom::Start(offset))
            .context(LogSeek {})?;

        let doc = Document::from_reader(&mut self.log_rd).context(Deser { offset })?;
        let found: Command = bson::from_bson(Bson::Document(doc)).context(Deser { offset })?;
        match found {
            Command::Set { key, val } => Ok(Some(val)),
            Command::Rm(_) => Err(Error::BadIndex {
                cmd: "Set".to_owned(),
                offset,
                found,
            }),
        }
    }

    /// Set the value for the specified key.
    ///
    /// If a value is already stored at this key it is unceremoniously overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let offset: u64 = self.pos;
        let cmd = Command::Set {
            key: key.clone(),
            val: value,
        };
        // seek until the end
        let pos = self
            .log_wr
            .seek(SeekFrom::End(0))
            .with_context(|| LogSeek {})?;
        let bs = bson::to_bson(&cmd).context(Ser { cmd })?;
        // We know its a document
        let doc = bs.as_document().unwrap();
        doc.to_writer(&mut self.log_wr)
            .context(LogWrite { offset })?;
        self.log_wr.flush().context(Io {
            action: "flush".to_owned(),
            offset,
        })?;

        self.pos = self.log_wr.seek(SeekFrom::End(0)).context(LogSeek {})?;
        self.index.insert(key.clone(), offset);
        assert!(self.index.contains_key(&key));
        Ok(())
    }

    /// Remove the value stored under the specified key.
    ///
    /// If nothing is stored at that key nothing happens.
    ///
    /// # Example
    /// ```rust
    /// # use kvs::KvStore;
    /// let mut store = KvStore::open("/tmp/log.kv")?;
    /// store.set("my key".to_owned(), "my value".to_owned())?;
    /// store.remove("my key".to_owned())?;
    /// let val = store.get("my key".to_owned())?;
    /// assert_eq!(val, None);
    /// ```
    pub fn remove(&mut self, key: String) -> Result<()> {
        unimplemented!();
    }
}
