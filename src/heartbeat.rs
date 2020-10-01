use crate::BoxedErrorResult;
use crate::component_manager;
use crate::component_manager::*;
use crate::constants;
use crate::globals;
use crate::packet::*;
use std::{error, io, thread, time};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

type HeartBeatResult = BoxedErrorResult<()>;

lazy_static! {
    pub static ref Count: Mutex<u64> = Mutex::new(0);
}

pub fn join(sender: &PacketSender) -> HeartBeatResult {
    // Add self to membership list and push the J packet onto the queue
    if is_joined() { return Ok(()) }

    globals::IS_JOINED.write(true);
    let my_id = gen_id();
    globals::MY_ID.write(my_id);
    // (*globals::MEMBERSHIP_LIST.get_mut()).unwrap().push(my_id); // TODO
    let queue_item = PacketQueueItem {
        dests: constants::IP_LIST.iter().cloned().collect(),
        packet: Box::new(JoinPacket {})
    };
    sender.send(queue_item)?;
    println!("sent join packet!");
    Ok(())
}

pub fn leave(sender: &PacketSender) -> HeartBeatResult {
    // 
    println!("left!");
    Ok(())
}

pub fn print() -> HeartBeatResult {
    println!("printed!");
    // let rpc_packet = packet::Packet::read(&UdpSocket::bind("127.0.0.1:34254")?);
    // rpc_packet.execute()?;
    Ok(())
}

fn gen_id() -> String {
    return format!("{}|{}", *globals::MY_IP_ADDR.read(), get_timestamp()).to_string();
}

fn get_timestamp() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}
