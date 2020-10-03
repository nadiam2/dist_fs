use crate::BoxedErrorResult;
use crate::component_manager;
use crate::component_manager::*;
use crate::constants;
use crate::globals;
use crate::operation::*;
use serde::{Serialize, Deserialize};
use std::{error, io, thread, time};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

type HeartBeatResult = BoxedErrorResult<()>;

lazy_static! {
    pub static ref Count: Mutex<u64> = Mutex::new(0);
}

pub fn join(sender: &OperationSender) -> HeartBeatResult {
    // Add self to membership list and push the J operation onto the queue
    if is_joined() { return Ok(()) }

    // Mark self as joined and in membership list
    globals::IS_JOINED.write(true);
    let my_id = gen_id();
    globals::MY_ID.write(my_id.clone());
    (*globals::MEMBERSHIP_LIST.get_mut()).push(my_id.clone());

    // Send Join operation to everyone
    let queue_item = OperationQueueItem {
        dests: constants::IP_LIST.iter().map(|x| x.to_string()).collect(),
        operation: Box::new(JoinOperation {id: my_id})
    };
    sender.send(queue_item)?;
    println!("sent join packet!");
    Ok(())
}

pub fn leave(sender: &OperationSender) -> HeartBeatResult {
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

// Operations
#[derive(Serialize, Deserialize, Debug)]
pub struct JoinOperation {
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewMemberOperation {
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MembershipListOperation {
    membership_list: Vec<String>
}

// Trait Impls
impl OperationWriteExecute for JoinOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['J' as u8]))
    }
    fn execute(&self, source: String, sender: &OperationSender) -> BoxedErrorResult<()> {
        // Add the new guy and send it to everyone
        insert_node(&self.id)?;
        sender.send(OperationQueueItem::for_everyone(Box::new(NewMemberOperation{
            id: self.id.clone()
        })))?;
        sender.send(OperationQueueItem::for_single(self.id.to_string(), Box::new(MembershipListOperation{
            membership_list: globals::MEMBERSHIP_LIST.read().clone()
        })))?;
        Ok(())
    }
}

impl OperationWriteExecute for NewMemberOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['N' as u8]))
    }
    fn execute(&self, source: String, _sender: &OperationSender) -> BoxedErrorResult<()> {
        insert_node(&self.id);
        // MAYBE TODO: Do we need more redundancy to make sure joins are not missed?
        Ok(())
    }
}

impl OperationWriteExecute for MembershipListOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['M' as u8]))
    }
    fn execute(&self, source: String, sender: &OperationSender) -> BoxedErrorResult<()> {
        merge_membership_list(&self.membership_list);
        Ok(())
    }
}
