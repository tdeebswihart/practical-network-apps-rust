use std::collections::HashMap;

/// A string to string key-value store
///
/// Key-value pairs are stored in memory and not persisted to disk.
///
/// Example usage:
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("my key".to_owned(), "my value".to_owned());
/// let val = store.get("my key".to_owned());
/// assert_eq!(val, Some("my value".to_owned()));
///```
pub struct KvStore {
    store: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> KvStore {
        KvStore {
            store: HashMap::new(),
        }
    }

    /// Retrieve the value stored at the specified key
    ///
    /// Returns `None` if the key does not exist.
    pub fn get(&self, key: String) -> Option<String> {
        match self.store.get(&key) {
            Some(v) => Some(v.to_owned()),
            None => None
        }
    }

    /// Set the value for the specified key.
    ///
    /// If a value is already stored at this key it is unceremoniously overwritten.
    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    /// Remove the value stored under the specified key.
    ///
    /// If nothing is stored at that key nothing happens.
    ///
    /// # Example
    /// ```rust
    /// # use kvs::KvStore;
    /// let mut store = KvStore::new();
    /// store.set("my key".to_owned(), "my value".to_owned());
    /// store.remove("my key".to_owned());
    /// let val = store.get("my key".to_owned());
    /// assert_eq!(val, None);
    /// ```
    pub fn remove(&mut self, key: String) {
        self.store.remove(&key);
    }
}
