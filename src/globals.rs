use crate::locks::*;
use std::fs::{File, OpenOptions};
use std::net::UdpSocket;

// Vars
lazy_static! {
    pub static ref DEBUG: RwLockOption<bool> = RwLockOption::new();
    pub static ref LOG_FILE: RwLockOption<File> = RwLockOption::new();
    pub static ref UDP_SOCKET: RwLockOption<UdpSocket> = RwLockOption::new();
    pub static ref IS_JOINED: RwLockOption<bool> = RwLockOption::new();
    pub static ref MEMBERSHIP_LIST: RwLockOption<Vec<String>> = RwLockOption::new();
    pub static ref SUCCESSOR_LIST: RwLockOption<Vec<String>> = RwLockOption::new();
    pub static ref PREDECESSOR_LIST: RwLockOption<Vec<String>> = RwLockOption::new();
    pub static ref MY_IP_ADDR: RwLockOption<String> = RwLockOption::new();
    pub static ref MY_ID: RwLockOption<String> = RwLockOption::new();
}
