use crate::locks::*;
use std::net::UdpSocket;

// Vars
lazy_static! {
    pub static ref UDP_SOCKET: RwLockOption<UdpSocket> = RwLockOption::new();
    pub static ref IS_JOINED: RwLockOption<bool> = RwLockOption::new();
    pub static ref MEMBERSHIP_LIST: RwLockOption<Vec<String>> = RwLockOption::new();
    pub static ref MY_IP_ADDR: RwLockOption<String> = RwLockOption::new();
    pub static ref MY_ID: RwLockOption<String> = RwLockOption::new();
}
