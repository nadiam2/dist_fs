use crate::heartbeat::Timestamp;

// House
// pub const IP_LIST: [&str; 4] = [
//     "192.168.86.70:9000",
//     "192.168.86.70:9001",
//     "192.168.86.70:9002",
//     "192.168.86.68:9000",
//     // when youre not lazy, get a setup of the machines working
// ];

// My Apartment
// pub const IP_LIST: [&str; 4] = [
//     "192.168.0.124:9000",
//     "192.168.0.124:9001",
//     "192.168.0.124:9002",
//     "192.168.0.174:9000",
//     // when youre not lazy, get a setup of the machines working
// ];

// Nad Apartment
pub const IP_LIST: [&str; 4] = [
    "192.168.10.12:9000",
    "192.168.10.12:9001",
    "192.168.10.12:9002",
    "192.168.10.49:9000",
];


pub static OP_TYPE_SIZE: usize = 4;
pub static HEADER_SIZE: usize = 8;
pub static NUM_SUCCESSORS: u32 = 2;
pub static NUM_OWNERS: u32 = 2;
pub static EXPIRATION_DURATION: Timestamp = 3;

pub static LOG_DIR: &str  = "logs";
pub static DATA_DIR: &str = "data";

// pub const IP_LIST: [&str; 4] = [
//     "localhost:9000",
//     "localhost:9001",
//     "localhost:9002",
//     "localhost:9003",
//     // when youre not lazy, get a setup of the machines working
// ];
