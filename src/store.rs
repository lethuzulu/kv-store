use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Write},
};

#[derive(Debug, Serialize, Deserialize)]
enum Command {
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

#[derive(Debug)]
pub struct KvStore {
    map: HashMap<String, Vec<u8>>,
    log: File,
}

impl KvStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        // let path = path.as_ref();

        let map = match Self::replay_log(&path) {
            Ok(m) => m,
            Err(_) => HashMap::new(),
        };

        if let Err(_) = Self::compact(&map, &path) { //TODO decide how & when to run compaction
            eprintln!("Error during compact.")
        }
        let log = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { map, log })
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let value = self.map.get(key);

        match value {
            Some(v) => return Some(v.clone()),
            None => return None,
        }
    }

    pub fn set<K: Into<String>, V: Into<Vec<u8>>>(&mut self, key: K, value: V) -> Result<()> {
        let key = key.into();
        let value = value.into();

        self.map.insert(key.clone(), value.clone());
        self.append_command(&Command::Set { key, value })?;
        Ok(())
    }

    pub fn delete<K: Into<String>>(&mut self, key: K) -> Result<()> {
        let key = key.into();

        self.map.remove(&key);
        self.append_command(&Command::Delete { key })?;
        Ok(())
    }

    fn append_command(&mut self, command: &Command) -> Result<()> {
        let mut line = serde_json::to_string(command)?;
        line.push('\n');
        self.log.write_all(line.as_bytes())?;
        Ok(())
    }

    fn replay_log(path: impl AsRef<Path>) -> Result<HashMap<String, Vec<u8>>> {
        let mut log = OpenOptions::new().read(true).open(path)?;

        let mut map = HashMap::new();
        let mut buf = String::new();

        log.read_to_string(&mut buf)?;

        for line in buf.lines() {
            let command = serde_json::from_str::<Command>(line)?;

            match command {
                Command::Set { key, value } => {
                    map.insert(key, value);
                }
                Command::Delete { key } => {
                    map.remove(&key);
                }
            }
        }

        Ok(map)
    }
    fn compact(map: &HashMap<String, Vec<u8>>, path: impl AsRef<Path>) -> Result<()> {
        let temporary_path = path.as_ref().with_extension("tmp");
        let mut temporary_log = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temporary_path)?;

        for (key, value) in map {
            let key = key.clone();
            let value = value.clone();

            let command = Command::Set { key, value };
            Self::append_compact_command(&command, &mut temporary_log)?
        }
        temporary_log.sync_all()?;
        std::fs::rename(temporary_path, path)?;
        Ok(())
    }

    fn append_compact_command(command: &Command, file: &mut File) -> Result<()> {
        let mut line = serde_json::to_string(command)?;
        line.push('\n');
        file.write_all(line.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::KvStore;
    use std::fs;

    //TODO use tempfile crate to generate tempfile instead of testing with real files
    fn setup(path: &str) -> KvStore {
        let _ = fs::remove_file(path);
        KvStore::new(path).unwrap()
    }

    fn cleanup(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_set_and_get() {
        let path = "test_set_and_get.log";
        let mut store = setup(path);

        store.set("key1", b"value1".to_vec()).unwrap();

        let result = store.get("key1");

        assert_eq!(result, Some(b"value1".to_vec()));

        cleanup(path);
    }

    #[test]
    fn test_delete() {
        let path = "test_delete.log";
        let mut store = setup(path);

        store.set("key1", b"value1".to_vec()).unwrap();
        store.delete("key1").unwrap();

        let result = store.get("key1");

        assert_eq!(result, None);

        cleanup(path);
    }

    #[test]
    fn test_overwrite() {
        let path = "test_overwrite.log";
        let mut store = setup(path);

        store.set("key1", b"value1".to_vec()).unwrap();
        store.set("key1", b"value2".to_vec()).unwrap();

        let result = store.get("key1");

        assert_eq!(result, Some(b"value2".to_vec()));

        cleanup(path);
    }

    #[test]
    fn test_persistence_after_restart() {
        let path = "test_persistence.log";
        let _ = fs::remove_file(path);

        {
            let mut store = KvStore::new(path).unwrap();
            store.set("key1", b"value1".to_vec()).unwrap();
        }

        // simulate restart
        let store = KvStore::new(path).unwrap();

        let result = store.get("key1");

        assert_eq!(result, Some(b"value1".to_vec()));

        cleanup(path);
    }

    #[test]
    fn test_multiple_keys() {
        let path = "test_multiple_keys.log";
        let mut store = setup(path);

        store.set("a", b"1".to_vec()).unwrap();
        store.set("b", b"2".to_vec()).unwrap();

        assert_eq!(store.get("a"), Some(b"1".to_vec()));
        assert_eq!(store.get("b"), Some(b"2".to_vec()));

        cleanup(path);
    }

    #[test]
    fn test_compaction_preserves_latest_values() {
        use tempfile::NamedTempFile;

        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        {
            let mut store = KvStore::new(path).unwrap();

            store.set("key", b"v1".to_vec()).unwrap();
            store.set("key", b"v2".to_vec()).unwrap();
            store.set("key", b"v3".to_vec()).unwrap();
        }

        // reopening triggers compaction
        let store = KvStore::new(path).unwrap();

        let result = store.get("key");

        assert_eq!(result, Some(b"v3".to_vec()));
    }

    #[test]
    fn test_compaction_removes_deleted_keys() {
        use tempfile::NamedTempFile;

        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        {
            let mut store = KvStore::new(path).unwrap();

            store.set("key", b"value".to_vec()).unwrap();
            store.delete("key").unwrap();
        }

        let store = KvStore::new(path).unwrap();

        let result = store.get("key");

        assert_eq!(result, None);
    }
    #[test]
    fn test_compaction_reduces_log_size() {
        use tempfile::NamedTempFile;
        use std::fs;

        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        {
            let mut store = KvStore::new(path).unwrap();

            for i in 0..100 {
                store.set("key", format!("value{}", i).into_bytes()).unwrap();
            }
        }

        let size_before = fs::metadata(path).unwrap().len();

        // reopening triggers compaction
        let _store = KvStore::new(path).unwrap();

        let size_after = fs::metadata(path).unwrap().len();

        assert!(size_after < size_before, "compaction did not reduce log size");
    }

    #[test]
    fn test_compacted_log_is_replayable() {
        use tempfile::NamedTempFile;

        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        {
            let mut store = KvStore::new(path).unwrap();

            store.set("a", b"1".to_vec()).unwrap();
            store.set("b", b"2".to_vec()).unwrap();
            store.delete("a").unwrap();
        }

        // triggers compaction
        let store = KvStore::new(path).unwrap();

        assert_eq!(store.get("a"), None);
        assert_eq!(store.get("b"), Some(b"2".to_vec()));
    }

}
