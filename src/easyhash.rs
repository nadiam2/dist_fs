use std::collections::hash_map::DefaultHasher;
use std::fmt::UpperHex;
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

pub trait Hex {
    fn hex(&self) -> String;
}

impl<T> Hex for T
where T: UpperHex {
    fn hex(&self) -> String {
        format!("{:X}", self)
    }
}
