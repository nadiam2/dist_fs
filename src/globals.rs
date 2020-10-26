use async_std;
use crate::heartbeat::Timestamp;
use crate::locks::*;
use std;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};

// Vars
lazy_static! {
    pub static ref DEBUG: RwLockOption<bool> = RwLockOption::new();
    pub static ref LOG_FILE: RwLockOption<File> = RwLockOption::new();
    pub static ref UDP_SOCKET: RwLockOption<std::net::UdpSocket> = RwLockOption::new();
    pub static ref IS_JOINED: RwLockOption<bool> = RwLockOption::new();
    pub static ref MEMBERSHIP_LIST: RwLockOption<Vec<String>> = RwLockOption::new();
    pub static ref SUCCESSOR_LIST: RwLockOption<Vec<String>> = RwLockOption::new();
    pub static ref PREDECESSOR_LIST: RwLockOption<Vec<String>> = RwLockOption::new();
    pub static ref PREDECESSOR_TIMESTAMPS: RwLockOption<HashMap<String, Timestamp>> = RwLockOption::new();
    pub static ref MY_IP_ADDR: RwLockOption<String> = RwLockOption::new();
    pub static ref MY_ID: RwLockOption<String> = RwLockOption::new();
    pub static ref SERVER_SOCKET: RwLockOption<async_std::net::TcpListener> = RwLockOption::new();
}
