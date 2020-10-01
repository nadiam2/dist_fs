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

    // Mark self as joined and in membership list
    globals::IS_JOINED.write(true);
    let my_id = gen_id();
    globals::MY_ID.write(my_id.clone());
    (*globals::MEMBERSHIP_LIST.get_mut()).push(my_id.clone());

    // Send Join packet to everyone
    let queue_item = PacketQueueItem {
        dests: constants::IP_LIST.iter().map(|x| x.to_string()).collect(),
        packet: Box::new(JoinPacket {id: my_id})
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
    let list = globals::MEMBERSHIP_LIST.read();
    println!("[");
    for memb in &*list {
        println!("  {}", memb);
    }
    println!("]");
    Ok(())
}

// Helpers
pub fn ips_from_ids(ids: Vec<String>) -> Vec<String> {
    ids.iter().map(|x| {
        let n = x.find('|').unwrap();
        String::from(&x[..n])
    }).collect()
}

pub fn insert_node(new_id: &String) -> HeartBeatResult {
    let membership_list = globals::MEMBERSHIP_LIST.read();
    if let Err(idx) = membership_list.binary_search(&new_id) {
        drop(membership_list);
        let mut membership_list = globals::MEMBERSHIP_LIST.get_mut();
        membership_list.insert(idx, new_id.to_string());
    }
    Ok(())
}

pub fn merge_membership_list(membership_list: &Vec<String>) -> HeartBeatResult {
    // TODO: Optimize?
    for member_id in membership_list {
        insert_node(member_id)?;
    }
    // TODO: Recalc relevant lists
    Ok(())
}

fn gen_id() -> String {
    return format!("{}|{}", *globals::MY_IP_ADDR.read(), get_timestamp()).to_string();
}

fn get_timestamp() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

