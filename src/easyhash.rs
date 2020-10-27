use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub trait EasyHash {
    fn easyhash(&self) -> u64;
}

impl<T> EasyHash for T
where T: Hash {
    fn easyhash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
