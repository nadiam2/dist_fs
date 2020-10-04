use crate::BoxedErrorResult;
use crate::component_manager::*;
use crate::constants;
use crate::globals;
use crate::modular::*;
use crate::operation::*;
use serde::{Serialize, Deserialize};
use std::time::SystemTime;

type HeartBeatResult = BoxedErrorResult<()>;

// TODO: Cleanup this bad boy
pub fn join(sender: &OperationSender) -> HeartBeatResult {
    if is_joined() {
        println!("Already joined");
        return Ok(());
    }
    // Mark self as joined and in membership list
    globals::IS_JOINED.write(true);
    let my_id = gen_id()?;
    globals::MY_ID.write(my_id.clone());
    globals::MEMBERSHIP_LIST.get_mut().push(my_id.clone());
    // Send Join operation to everyone
    let join_item = SendableOperation {
        dests: constants::IP_LIST.iter().map(|x| x.to_string()).collect(),
        operation: Box::new(JoinOperation {id: my_id.clone()})
    };
    sender.send(join_item)?;
    log(format!("Joined the network with id: {}", my_id));
    Ok(())
}

pub fn leave(sender: &OperationSender) -> HeartBeatResult {
    if !is_joined() {
        println!("Not joined yet");
        return Ok(());
    }
    // Send the leave operation and clear vars
    let my_id = globals::MY_ID.read().clone();
    let leave_item = SendableOperation::for_successors(Box::new(LeaveOperation{
        id: my_id.clone()
    }));
    sender.send(leave_item)?;
    clear_vars_on_leave();
    log(format!("Left the network with id: {}", my_id));
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

pub fn send_heartbeats() -> HeartBeatResult {
    let udp_socket = globals::UDP_SOCKET.read();
    let successor_list = globals::SUCCESSOR_LIST.read().clone();
    let heartbeat_operation = Box::new(HeartbeatOperation {});
    let heartbeats = SendableOperation::for_successors(heartbeat_operation);
    heartbeats.write_all(&udp_socket)
}

// Helpers
pub fn ips_from_ids(ids: Vec<String>) -> Vec<String> {
    ids.iter().map(|x| {
        let n = x.find('|').unwrap();
        String::from(&x[..n])
    }).collect()
}

pub fn insert_node(new_id: &String) -> HeartBeatResult {
    let mut membership_list = globals::MEMBERSHIP_LIST.get_mut();
    if let Err(idx) = membership_list.binary_search(&new_id) {
        membership_list.insert(idx, new_id.to_string());
    }
    Ok(())
}

pub fn remove_node(id: &String) -> BoxedErrorResult<bool> {
    let mut membership_list = globals::MEMBERSHIP_LIST.get_mut();
    match membership_list.binary_search(&id).clone() {
        Ok(idx) => {
            membership_list.remove(idx);
            Ok(true)
        }
        Err(_) => Ok(false)
    }
}

fn clear_vars_on_leave() -> BoxedErrorResult<()> {
    globals::IS_JOINED.write(false);
    globals::MEMBERSHIP_LIST.get_mut().clear();
    globals::SUCCESSOR_LIST.get_mut().clear();
    globals::PREDECESSOR_LIST.get_mut().clear();
    // Does keeping the old id matter? If so, edit the locks to be able to write None back in
    Ok(())
}

pub fn merge_membership_list(membership_list: &Vec<String>) -> HeartBeatResult {
    // TODO: Optimize?
    for member_id in membership_list {
        insert_node(member_id)?;
    }
    Ok(())
}

pub fn recalculate_neighbors() -> HeartBeatResult {
    recalculate_predecessors()?;
    recalculate_successors()?;
    Ok(())
}

fn recalculate_successors() -> HeartBeatResult {
    let successors = gen_neighbor_list(1)?;
    log(format!("Calculated successors as {:?}", successors));
    globals::SUCCESSOR_LIST.write(successors);
    Ok(())
}

fn recalculate_predecessors() -> HeartBeatResult {
    let predecessors = gen_neighbor_list(-1)?;
    log(format!("Calculated predecessors as {:?}", predecessors));
    globals::PREDECESSOR_LIST.write(predecessors);
    Ok(())
}

// TODO: Eventually change this code to make sure the data fits
fn gen_neighbor_list(increment: i32) -> BoxedErrorResult<Vec<String>> {
    // Ensure the membership_list has members
    let membership_list = globals::MEMBERSHIP_LIST.read();
    let len = membership_list.len();
    if len == 0 {
        return Err("Cannot calculate successors/predecessors because membership list is empty".into())
    }
    // Traverse modulo membership list size
    let my_idx: usize = membership_list.binary_search(&*globals::MY_ID.read())
        .expect("ID of node not found in membership list");
    let start_idx = Modular::new(my_idx as i32 + 1, len as u32);
    let mut new_neighbors = Vec::new();
    for i in 0..constants::NUM_SUCCESSORS {
        let curr_idx = start_idx.clone() + i;
        if *curr_idx == my_idx as u32 {
            break
        }
        new_neighbors.push(membership_list[*curr_idx as usize].clone());
    }
    Ok(new_neighbors)
}

fn gen_id() -> BoxedErrorResult<String> {
    Ok(format!("{}|{}", *globals::MY_IP_ADDR.read(), get_timestamp()?).to_string())
}

pub fn get_timestamp() -> BoxedErrorResult<u64> {
    Ok(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs())
}

// Operations
#[derive(Serialize, Deserialize, Debug)]
pub struct HeartbeatOperation {}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinOperation {
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaveOperation {
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
impl OperationWriteExecute for HeartbeatOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['H' as u8]))
    }
    fn execute(&self, _source: String, sender: &OperationSender) -> BoxedErrorResult<()> {
        // Update the most recent observation
        Ok(())
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}

impl OperationWriteExecute for JoinOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['J' as u8]))
    }
    fn execute(&self, _source: String, sender: &OperationSender) -> BoxedErrorResult<()> {
        // Add the new guy and send it to everyone
        insert_node(&self.id)?;
        sender.send(SendableOperation::for_everyone(Box::new(NewMemberOperation{
            id: self.id.clone()
        })))?;
        sender.send(SendableOperation::for_single(self.id.to_string(), Box::new(MembershipListOperation{
            membership_list: globals::MEMBERSHIP_LIST.read().clone()
        })))?;
        recalculate_neighbors()?;
        Ok(())
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}


impl OperationWriteExecute for LeaveOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['L' as u8]))
    }
    fn execute(&self, _source: String, sender: &OperationSender) -> BoxedErrorResult<()> {
        let removed = remove_node(&self.id)?;
        if removed {
            sender.send(SendableOperation::for_successors(Box::new(LeaveOperation{
                id: self.id.clone()
            })))?;
            recalculate_neighbors()?;
        }
        Ok(())
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}

impl OperationWriteExecute for NewMemberOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['N' as u8]))
    }
    fn execute(&self, _source: String, _sender: &OperationSender) -> BoxedErrorResult<()> {
        insert_node(&self.id)?;
        // MAYBE TODO: Do we need more redundancy to make sure joins are not missed?
        recalculate_neighbors()?;
        Ok(())
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}

impl OperationWriteExecute for MembershipListOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['M' as u8]))
    }
    fn execute(&self, _source: String, _sender: &OperationSender) -> BoxedErrorResult<()> {
        merge_membership_list(&self.membership_list)?;
        recalculate_neighbors()?;
        Ok(())
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}
