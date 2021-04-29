use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

// This file provides easy global access to RwLocks and Mutex guarded variables
// The general strategy is that
// new: initializes the inner option to be none
// write: lets you write Some(val) directly into the option
// read: lets you get an immutable reference which can be derefed for accessing the value
// get_mut: lets you get a mutable reference which can be derefed for accessing/modifying the value

// Lock Shenanigans
pub struct RwLockOption<T> {
    lock: RwLock<Option<T>>
}

impl<T> RwLockOption<T> {
    pub fn new() -> Self {
        RwLockOption{lock: RwLock::new(None)}
    }
    pub fn read<'a>(&'a self) -> InnerRwReadLock<'a, T> {
        let guard = self.lock.read().unwrap();
        InnerRwReadLock{guard}
    }
    pub fn write(&self, val: T) {
        let mut opt = self.lock.write().unwrap();
        *opt = Some(val);
    }
    pub fn get_mut<'a>(&'a self) -> InnerRwWriteLock<'a, T> {
        let guard = self.lock.write().unwrap();
        InnerRwWriteLock{guard}
    }
}

pub struct InnerRwReadLock<'a, T> {
    guard: RwLockReadGuard<'a, Option<T>>
}

pub struct InnerRwWriteLock<'a, T> {
    guard: RwLockWriteGuard<'a, Option<T>>
}

impl<'a, T> Deref for InnerRwReadLock<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

impl<'a, T> Deref for InnerRwWriteLock<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for InnerRwWriteLock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_mut().unwrap()
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

impl<'a, T> DerefMut for InnerMutex<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_mut().unwrap()
    }
}
