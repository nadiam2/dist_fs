use std::ops::Deref;
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard};

// Lock Shenanigans
pub struct RwLockOption<T> {
    lock: RwLock<Option<T>>
}

impl<T> RwLockOption<T> {
    pub fn new() -> Self {
        RwLockOption{lock: RwLock::new(None)}
    }
    pub fn read<'a>(&'a self) -> InnerRwLock<'a, T> {
        let guard = self.lock.read().unwrap();
        InnerRwLock{guard}
    }
    pub fn write(&self, val: T) {
        let mut opt = self.lock.write().unwrap();
        *opt = Some(val);
    }
}

pub struct InnerRwLock<'a, T> {
    guard: RwLockReadGuard<'a, Option<T>>
}

impl<'a, T> Deref for InnerRwLock<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

pub struct MutexOption<T> {
    lock: Mutex<Option<T>>
}

impl<T> MutexOption<T> {
    pub fn new() -> Self {
        MutexOption{lock: Mutex::new(None)}
    }
    pub fn read<'a>(&'a self) -> InnerMutex<'a, T> {
        let guard = self.lock.lock().unwrap();
        InnerMutex{guard}
    }
    pub fn write(&self, val: T) {
        let mut opt = self.lock.lock().unwrap();
        *opt = Some(val);
    }
}

pub struct InnerMutex<'a, T> {
    guard: MutexGuard<'a, Option<T>>
}

impl<'a, T> Deref for InnerMutex<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}
