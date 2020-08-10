use snafu::{ResultExt, Snafu};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("io error writing to {}: {}", path.display(), source))]
    Write { source: io::Error, path: PathBuf },
    #[snafu(display("io error reading from {}: {}", path.display(), source))]
    Read { source: io::Error, path: PathBuf },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A string to string key-value store
///
/// Key-value pairs are stored in memory and not persisted to disk.
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
    store: HashMap<String, String>,
}

impl KvStore {
    pub fn open(_path: impl Into<PathBuf>) -> Result<KvStore> {
        unimplemented!();
    }

    /// Retrieve the value stored at the specified key
    ///
    /// Returns `None` if the key does not exist.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(match self.store.get(&key) {
            Some(v) => Some(v.to_owned()),
            None => None,
        })
    }

    /// Set the value for the specified key.
    ///
    /// If a value is already stored at this key it is unceremoniously overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.store.insert(key, value);
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
        self.store.remove(&key);
        Ok(())
    }
}
