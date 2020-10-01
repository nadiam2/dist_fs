use crate::BoxedErrorResult;
use crate::component_manager;
use crate::component_manager::{is_joined, PacketSender, PacketReceiver};
use crate::constants;
use crate::packet::*;
use std::{error, io, thread, time};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};

type HeartBeatResult = BoxedErrorResult<()>;

lazy_static! {
    pub static ref Count: Mutex<u64> = Mutex::new(0);
}

pub fn join(sender: &PacketSender) -> HeartBeatResult {
    // Add self to membership list and push the J packet onto the queue
    if is_joined() { return Ok(()) }

    component_manager::IS_JOINED_LOCK.write(true);
    let queue_item = PacketQueueItem {
        dests: constants::IP_LIST.iter().cloned().collect(),
        packet: Packet::Join(Box::new(JoinPacket {}))
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

// Err(String::from("yo").into())
