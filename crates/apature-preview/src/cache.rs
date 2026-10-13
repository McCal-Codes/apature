//! On-disk thumbnail cache. Keys are a hash of (path, size, mtime, long side),
//! so edits or re-ingests invalidate automatically. Values are JPEGs.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::decode::{decode_jpeg, encode_jpeg, Rgb};

#[derive(Clone, Debug)]
pub struct ThumbCache {
    dir: PathBuf,
}

impl ThumbCache {
    pub fn new(dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(ThumbCache { dir })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Cache key for a file. `modified`/`size` come from the directory scan so
    /// no extra stat is needed.
    pub fn key(path: &Path, size: u64, modified: Option<SystemTime>, long_side: u32) -> String {
        let mut h = DefaultHasher::new();
        path.hash(&mut h);
        size.hash(&mut h);
        modified
            .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0)
            .hash(&mut h);
        long_side.hash(&mut h);
        format!("{:016x}-{long_side}", h.finish())
    }

    fn path_for(&self, key: &str) -> PathBuf {
        // Two-level fan-out keeps directories small with tens of thousands of entries.
        self.dir.join(&key[..2]).join(format!("{key}.jpg"))
    }

    pub fn get(&self, key: &str) -> Option<Rgb> {
        let bytes = fs::read(self.path_for(key)).ok()?;
        decode_jpeg(&bytes, None).ok()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.path_for(key).is_file()
    }

    pub fn put(&self, key: &str, img: &Rgb) -> io::Result<()> {
        let bytes = encode_jpeg(img, 88).map_err(io::Error::other)?;
        let final_path = self.path_for(key);
        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = final_path.with_extension(format!("tmp{}", std::process::id()));
        fs::write(&tmp, bytes)?;
        fs::rename(tmp, final_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_then_get() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ThumbCache::new(dir.path().join("thumbs")).unwrap();
        let img = Rgb {
            width: 4,
            height: 2,
            data: vec![200; 24],
        };
        let key = ThumbCache::key(Path::new("/a/b.NEF"), 10, None, 320);
        assert!(!cache.contains(&key));
        cache.put(&key, &img).unwrap();
        assert!(cache.contains(&key));
        let back = cache.get(&key).unwrap();
        assert_eq!((back.width, back.height), (4, 2));
        let other = ThumbCache::key(Path::new("/a/b.NEF"), 11, None, 320);
        assert_ne!(key, other);
    }
}
