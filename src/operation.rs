use bincode;
use crate::BoxedErrorResult;
use crate::component_manager::{log, OperationSender};
use crate::globals;
use crate::heartbeat::{ips_from_ids, JoinOperation, NewMemberOperation, MembershipListOperation};
use serde::{Serialize};
use std::convert::TryInto;
use std::fmt::Debug;
use std::net::UdpSocket;


// OPCODES
// 'J': Join -> empty packet to introducer
// 'L': Leave -> data should contain node id of node leaving
// 'M': Membership List -> data should be list itself
//      and have listener update their list (usually will be initializing it)
// 'N': New Node -> data should contain id of new node

// Constants
static HEADER_SIZE: usize = 5;

// Types
type BoxedOperation = Box<dyn OperationWriteExecute + Send + Sync>;

// Operation Queue Item
pub struct OperationQueueItem {
    pub dests: Vec<String>,
    pub operation: BoxedOperation
}

impl OperationQueueItem {
    pub fn write_all(&self, socket: &UdpSocket) -> BoxedErrorResult<()> {
        let serialized = self.operation.to_bytes()?;
        for dest in &self.dests {
            socket.send_to(&serialized, &dest)?;
        }
        log(format!("Sending a {} to {:?}", self.operation.to_string(), self.dests));
        Ok(())
    }
    pub fn for_list(dest_ids: Vec<String>, operation: BoxedOperation) -> Self {
        OperationQueueItem{
            dests: ips_from_ids(dest_ids),
            operation: operation
        }
    }
    pub fn for_everyone(operation: BoxedOperation) -> Self {
        Self::for_list((*globals::MEMBERSHIP_LIST.read()).clone(), operation)
    }
    pub fn for_single(dest_id: String, operation: BoxedOperation) -> Self {
        Self::for_list(vec![dest_id], operation)
    }
}

// Traits
pub trait OperationWriteExecute {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>>;
    fn execute(&self, source: String, sender: &OperationSender) -> BoxedErrorResult<()>;
    fn to_string(&self) -> String;
}

// Functions
pub fn read_operation(socket: &UdpSocket) -> BoxedErrorResult<(BoxedOperation, String)> {
    // Parse the header
    let mut header: Vec<u8> = vec![0; HEADER_SIZE];
    let _ = socket.peek_from(&mut header).expect("Read called on an empty socket.");
    let buf_size: u32 = u32::from_le_bytes(header[1..5].try_into().unwrap());
    // Receive the full message
    let mut buf: Vec<u8> = vec![0; buf_size as usize];
    let (_, sender) = socket.recv_from(&mut buf)
        .expect("Read called on an empty socket.");
    // Create the correct operation
    let operation: BoxedOperation = match buf[0] as char {
        'J' => Box::new(bincode::deserialize::<JoinOperation>(&buf[HEADER_SIZE..]).unwrap()),
        'N' => Box::new(bincode::deserialize::<NewMemberOperation>(&buf[HEADER_SIZE..]).unwrap()),
        'M' => Box::new(bincode::deserialize::<MembershipListOperation>(&buf[HEADER_SIZE..]).unwrap()),
        _   => return Err(String::from("Read unrecognized operation header").into())
    };
    
    log(format!("Read a {} from {:?}", operation.to_string(), &sender));
    return Ok((operation, sender.to_string()));
}

pub fn create_buf<T>(obj: &T, mut base: Vec<u8>) -> Vec<u8>
where T: Serialize {
    let serialized = bincode::serialize(obj).unwrap();
    let size: u32 = (5 + serialized.len()) as u32;
    base.extend_from_slice(&size.to_le_bytes());
    base.extend_from_slice(&serialized);
    return base;
}
