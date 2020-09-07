#[macro_use]
extern crate log;

use bson::de::Error as BsonDeError;
use bson::ser::Error as BsonSerError;
use bson::{Bson, Document};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to create directory {}: {}", path.display(), source))]
    MkDir { source: io::Error, path: PathBuf },
    #[snafu(display("failed to replay epoch {}: {}", epoch, source))]
    Replay {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<Error>,
        epoch: u64,
    },
    #[snafu(display("failed to compact log: {}", source))]
    Compact {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<Error>,
    },
    #[snafu(display("failed to remove outdated log {}: {}", epoch, source))]
    RemoveLog { source: io::Error, epoch: u64 },
    #[snafu(display("failed to open {}: {}", path.display(), source))]
    Open { source: io::Error, path: PathBuf },

    #[snafu(display("failed to list {}: {}", path.display(), source))]
    ListDir { source: io::Error, path: PathBuf },
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
    #[snafu(display("Key not found"))]
    NotFound,
    #[snafu(display("Expected command {} at offset {}, found {:?}", cmd, offset, found))]
    BadIndex {
        cmd: String,
        offset: u64,
        found: Command,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Log alteration commands.
#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Set { key: String, val: String },
    Rm(String),
}

struct LogFile {
    epoch: u64,
    handle: File,
    pos: u64,
}

impl io::Read for LogFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.handle.read(buf)?;
        self.pos += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl io::Write for LogFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let wrote = self.handle.write(buf)?;
        self.pos += wrote as u64;
        Ok(wrote)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.handle.flush()
    }
}

impl io::Seek for LogFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let pos = self.handle.seek(pos)?;
        self.pos = pos;
        Ok(pos)
    }
}

impl LogFile {
    fn new(epoch: u64, path: impl Into<PathBuf>) -> io::Result<LogFile> {
        let mut path = path.into();
        path.push(epoch.to_string());
        let mut handle = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;
        let length = handle.seek(SeekFrom::End(0))?;

        Ok(LogFile {
            epoch,
            handle,
            pos: length,
        })
    }

    /// Read the command, if any, stored at the provided offset.
    fn retrieve(&mut self, offset: u64) -> Result<Command> {
        self.seek(SeekFrom::Start(offset))
            .with_context(|| LogSeek {})?;
        let doc = Document::from_reader(self).context(Deser { offset })?;
        let found: Command = bson::from_bson(Bson::Document(doc)).context(Deser { offset })?;

        debug!("read {:?} in epoch {}@{}", &found, self.epoch, offset);
        Ok(found)
    }

    /// Record a command to the log file.
    ///
    /// Returns the offset from the start of the file the command was written to.
    fn record(&mut self, cmd: Command) -> Result<u64> {
        debug!("recording {:?} in epoch {}@{}", &cmd, self.epoch, self.pos);
        let bs = bson::to_bson(&cmd).context(Ser { cmd })?;
        // We know its a document
        let doc = bs.as_document().unwrap();
        let offset = self.seek(SeekFrom::End(0)).with_context(|| LogSeek {})?;
        doc.to_writer(self).context(LogWrite { offset })?;
        self.flush().context(Io {
            action: "flush".to_owned(),
            offset,
        })?;
        debug!("recorded command in epoch {}@{}", self.epoch, offset);
        Ok(offset)
    }

    // Replay the log, applying a callback function to every recorded event.
    fn replay<F: FnMut(Command, u64)>(&mut self, mut callback: F) -> Result<()> {
        debug!("replaying epoch {}", self.epoch);
        let length = self.seek(SeekFrom::End(0)).with_context(|| LogSeek {})?;
        self.seek(SeekFrom::Start(0)).with_context(|| LogSeek {})?;
        while self.pos < length {
            let offset = self.pos;
            let cmd = self.retrieve(self.pos)?;
            callback(cmd, offset);
        }
        Ok(())
    }
}

struct KeyEntry {
    epoch: u64,
    offset: u64,
}

type KeyDir = HashMap<String, KeyEntry>;

const DEFAULT_MAX_LOG_SIZE: u64 = 10_000_000; // 10MB

/// A string to string key-value store
///
/// Key-value pairs are stored in a single log file on disk.
///
/// Example usage:
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::open("/tmp/logd").expect("should work");
/// store.set("my key".to_owned(), "my value".to_owned());
/// let val = store.get("my key".to_owned()).expect("should exist");
/// assert_eq!(val, Some("my value".to_owned()));
///```
pub struct KvStore {
    // TODO: keep multiple log files?
    index: KeyDir,
    path: PathBuf,
    // Writer for the current epoch
    log: LogFile,
    // TODO: keep an LRU cache of file handles, keyed by epoch?
    // readers: HashMap<u64, File>
    epoch: u64,
    max_log_size: u64,
    // These tests require us to trigger compaction. I'd rather push that up to another layer, but to get it over with
    // we'll trigger compaction after every 100 removals or overwrites.
    mutations: u64,
}

impl KvStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();

        if let Err(e) = fs::create_dir_all(&path) {
            if e.kind() != io::ErrorKind::AlreadyExists {
                return Err(e).context(MkDir { path });
            }
        };

        let mut index = KeyDir::new();
        let mut logs = Vec::<LogFile>::new();

        for entry in fs::read_dir(&path).context(ListDir { path: path.clone() })? {
            let f = entry.expect("failed to list direntry");

            // There is probably an easier way to do this...
            let e: u64 = f
                .file_name()
                .to_string_lossy()
                .parse()
                .expect("expected file to have a u64 name");
            let lf = LogFile::new(e, path.clone()).with_context(|| Open { path: f.path() })?;
            logs.push(lf);
        }

        // Sort from lowest to highest epoch
        logs.sort_unstable_by(|a, b| a.epoch.partial_cmp(&b.epoch).unwrap());

        let mut epoch: u64 = 0;
        for log in &mut logs {
            epoch = log.epoch;
            log.replay(|cmd: Command, offset: u64| {
                match cmd {
                    Command::Set { key, val: _ } => {
                        index.insert(key, KeyEntry { epoch, offset });
                    }
                    Command::Rm(key) => {
                        index.remove(&key);
                    }
                };
            })
            .context(Replay { epoch })?;
        }

        // Grab file for the current epoch
        let log: LogFile = if logs.len() == 0 {
            LogFile::new(epoch, path.clone()).with_context(|| Open { path: path.clone() })?
        } else {
            logs.pop().unwrap()
        };

        Ok(KvStore {
            index,
            path,
            log,
            epoch,
            max_log_size: DEFAULT_MAX_LOG_SIZE,
            mutations: 0,
        })
    }

    // TODO: the KvStore should either take a callback that defines when to compact, or should only compact manually.
    fn should_compact(&self) -> bool {
        return self.mutations > 1000;
    }

    /// Set the size after which the store will rotate to a new log file.
    pub fn with_max_size(mut self, max_log_size: u64) -> Self {
        self.max_log_size = max_log_size;
        self
    }

    /// Retrieve the value stored at the specified key
    ///
    /// Returns `None` if the key does not exist.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if !self.index.contains_key(&key) {
            return Ok(None);
        }
        // Otherwise seek and get the key
        let entry = self.index.get(&key).unwrap().clone();

        debug!("getting {} from {}@{}", &key, entry.epoch, entry.offset);
        if entry.epoch == self.epoch {
            return match self.log.retrieve(entry.offset)? {
                Command::Set { key: k2, val } => {
                    debug_assert!(key == k2, "found a set for the wrong key");
                    Ok(Some(val))
                }
                Command::Rm(k) => Err(Error::BadIndex {
                    cmd: "Set".to_owned(),
                    offset: entry.offset,
                    found: Command::Rm(k),
                }),
            };
        }

        // TODO cache log handles?
        let mut log = LogFile::new(entry.epoch, self.path.clone()).with_context(|| Open {
            path: self.path.clone(),
        })?;

        match log.retrieve(entry.offset)? {
            Command::Set { key: k2, val } => {
                debug_assert!(key == k2, "found a set for the wrong key");
                Ok(Some(val))
            }
            Command::Rm(k) => Err(Error::BadIndex {
                cmd: "Set".to_owned(),
                offset: entry.offset,
                found: Command::Rm(k),
            }),
        }
    }

    /// Set the value for the specified key.
    ///
    /// If a value is already stored at this key it is unceremoniously overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::Set {
            key: key.clone(),
            val: value,
        };

        let offset = self.log.record(cmd)?;
        let previous = self.index.insert(
            key,
            KeyEntry {
                epoch: self.epoch,
                offset,
            },
        );

        if previous.is_some() {
            self.mutations += 1;
            if self.should_compact() {
                return self.compact().context(Compact);
            }
        }

        if self.log.pos < self.max_log_size {
            return Ok(());
        }

        // New epoch
        self.epoch += 1;
        debug!("beginning epoch {}", self.epoch);
        self.log = LogFile::new(self.epoch, self.path.clone()).with_context(|| Open {
            path: self.path.clone(),
        })?;

        Ok(())
    }

    /// Iterate over the log files from newest to oldest, keeping the full KV map in memory
    /// Once it reaches a certain size, write to disk as a new epoch
    pub fn compact(&mut self) -> Result<()> {
        self.epoch += 1;
        let rm_until = self.epoch;
        self.log = LogFile::new(self.epoch, self.path.clone()).with_context(|| Open {
            path: self.path.clone(),
        })?;

        let keys: Vec<String> = self.index.keys().map(|k| k.clone()).collect();

        for key in keys {
            if let Some(value) = self.get(key.clone())? {
                // May rotate to a new log file. That's fine!
                // prevent nested compaction.
                self.mutations = 0;
                self.set(key, value)?;
            };
        }

        for entry in fs::read_dir(&self.path).with_context(|| ListDir {
            path: self.path.clone(),
        })? {
            let f = entry.expect("failed to list direntry");

            // There is probably an easier way to do this...
            let e: u64 = f
                .file_name()
                .to_string_lossy()
                .parse()
                .expect("expected file to have a u64 name");
            if e < rm_until {
                // remove the file
                fs::remove_file(f.path()).context(RemoveLog { epoch: e })?;
            }
        }
        Ok(())
    }

    /// Remove the value stored under the specified key.
    ///
    /// If nothing is stored at that key nothing happens.
    ///
    /// # Example
    /// ```rust
    /// # use kvs::KvStore;
    /// let mut store = KvStore::open("/tmp/logd").expect("should open");
    /// store.set("my key".to_owned(), "my value".to_owned());
    /// store.remove("my key".to_owned());
    /// let val = store.get("my key".to_owned()).expect("shouldn't error");
    /// assert_eq!(val, None);
    /// ```
    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.index.contains_key(&key) {
            return Err(Error::NotFound);
        }
        let cmd = Command::Rm(key.clone());
        self.log.record(cmd)?;
        self.index.remove(&key);

        self.mutations += 1;
        if self.should_compact() {
            return self.compact().context(Compact);
        }
        Ok(())
    }
}
