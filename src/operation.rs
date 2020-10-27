use async_std;
use async_std::io::ReadExt;
use async_std::prelude::*;
use async_trait::async_trait;
use bincode;
use crate::BoxedErrorResult;
use crate::component_manager::{log, OperationSender};
use crate::constants::{HEADER_SIZE, OP_TYPE_SIZE};
use crate::filesystem::{GetOperation, NewFileOperation, SendFileOperation};
use crate::globals;
use crate::heartbeat::{ips_from_ids, HeartbeatOperation, JoinOperation, LeaveOperation, NewMemberOperation, MembershipListOperation, self};
use serde::{Serialize};
use std::convert::TryInto;
use std::fmt::Debug;
use std::net::UdpSocket;

// Types
type BoxedOperation = Box<dyn OperationWriteExecute + Send + Sync>;

// Operation Queue Item
pub struct SendableOperation {
    pub udp_dests: Vec<String>,
    pub operation: BoxedOperation
}

impl SendableOperation {
    // TODO: Maybe change these two to trait impls on the socket/stream
    pub fn write_all_udp(self, socket: &UdpSocket) -> BoxedErrorResult<()> {
        let serialized = self.operation.to_bytes()?;
        for udp_dest in &self.udp_dests {
            socket.send_to(&serialized, &udp_dest)?;
        }
        log(format!("Sent a {} to {:?}", self.operation.to_string(), self.udp_dests));
        Ok(())
    }
    pub async fn write_all_tcp_async(self) -> BoxedErrorResult<Vec<async_std::net::TcpStream>> {
        let serialized = self.operation.to_bytes()?;
        let tcp_map = globals::UDP_TO_TCP_MAP.read();
        // Collect for logging purposes
        let tcp_dests = heartbeat::tcp_ips_from_udp_ips(&self.udp_dests)?;
        let mut streams: Vec<async_std::net::TcpStream> = Vec::new();
        // TODO: Parallelize
        for dest in &tcp_dests {
            let mut stream = async_std::net::TcpStream::connect(dest).await?;
            streams.push(stream.clone());
            stream.write_all(&serialized).await?;
        }
        log(format!("Sent a {} to {:?}", self.operation.to_string(), tcp_dests));
        Ok(streams)        
    }
    pub fn for_id_list(dest_ids: Vec<String>, operation: BoxedOperation) -> Self {
        SendableOperation{
            udp_dests: ips_from_ids(&dest_ids),
            operation: operation
        }
    }
    pub fn for_everyone(operation: BoxedOperation) -> Self {
        Self::for_id_list(globals::MEMBERSHIP_LIST.read().clone(), operation)
    }
    pub fn for_single(dest_id: String, operation: BoxedOperation) -> Self {
        Self::for_id_list(vec![dest_id], operation)
    }
    pub fn for_successors(operation: BoxedOperation) -> Self {
        Self::for_id_list(globals::SUCCESSOR_LIST.read().clone(), operation)
    }
}

// Traits
pub trait OperationWriteExecute {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>>;
    fn execute(&self, source: String) -> BoxedErrorResult<Vec<SendableOperation>>;
    fn to_string(&self) -> String;
}

// Functions
pub fn create_buf<T>(obj: &T, mut base: Vec<u8>) -> Vec<u8>
where T: Serialize {
    let serialized = bincode::serialize(obj).unwrap();
    let size: u32 = (HEADER_SIZE + serialized.len()) as u32;
    base.extend_from_slice(&size.to_le_bytes());
    base.extend_from_slice(&serialized);
    return base;
}

pub trait TryReadOperation {
    fn try_read_operation(&self) -> BoxedErrorResult<(BoxedOperation, String)>;
}

#[async_trait]
pub trait TryReadOperationAsync {
    async fn try_read_operation(&mut self) -> BoxedErrorResult<(BoxedOperation, String)>;
}

// If s contains unicode, this is screwed. So don't do that :)
pub fn str_to_vec(s: &str) -> Vec<u8> {
    s.chars().map(|c| c as u8).collect()
}

pub fn vec_to_str(v: &Vec<u8>) -> String {
    v[..OP_TYPE_SIZE].iter().map(|b| b.clone() as char).collect::<String>()
}

fn try_parse_buf(buf: &Vec<u8>) -> BoxedErrorResult<BoxedOperation> {
    let operation: BoxedOperation = match vec_to_str(&buf).as_str() {
        "HB  " => Box::new(bincode::deserialize::<HeartbeatOperation>(&buf[HEADER_SIZE..]).unwrap()),
        "JOIN" => Box::new(bincode::deserialize::<JoinOperation>(&buf[HEADER_SIZE..]).unwrap()),
        "LEAV" => Box::new(bincode::deserialize::<LeaveOperation>(&buf[HEADER_SIZE..]).unwrap()),
        "NMEM" => Box::new(bincode::deserialize::<NewMemberOperation>(&buf[HEADER_SIZE..]).unwrap()),
        "MLIS" => Box::new(bincode::deserialize::<MembershipListOperation>(&buf[HEADER_SIZE..]).unwrap()),
        "GET " => Box::new(bincode::deserialize::<GetOperation>(&buf[HEADER_SIZE..]).unwrap()),
        "NFIL" => Box::new(bincode::deserialize::<NewFileOperation>(&buf[HEADER_SIZE..]).unwrap()),
        "FILE" => Box::new(bincode::deserialize::<SendFileOperation>(&buf[HEADER_SIZE..]).unwrap()),
        _   => return Err(String::from("Read unrecognized operation header").into())
    };
    Ok(operation)
}

impl TryReadOperation for UdpSocket {
    fn try_read_operation(&self) -> BoxedErrorResult<(BoxedOperation, String)> {
        // Parse the header
        let mut header: Vec<u8> = vec![0; HEADER_SIZE];
        let _ = self.peek_from(&mut header).expect("Read called on an empty UDP socket.");
        let buf_size: u32 = u32::from_le_bytes(header[OP_TYPE_SIZE..OP_TYPE_SIZE+4].try_into()?);
        // Receive the full message - TODO: Some assertions on the buf_size before creating the vec?
        let mut buf: Vec<u8> = vec![0; buf_size as usize];
        let (_, sender) = self.recv_from(&mut buf)
            .expect("Read called on an empty UDP socket.");
        // Create the correct operation
        let operation = try_parse_buf(&buf)?;
        log(format!("Read a {} from {:?}", operation.to_string(), &sender));
        return Ok((operation, sender.to_string()));
    }
}

#[async_trait]
impl TryReadOperationAsync for async_std::net::TcpStream {
    async fn try_read_operation(&mut self) -> BoxedErrorResult<(BoxedOperation, String)> {
        // Parse the header
        let mut header: Vec<u8> = vec![0; HEADER_SIZE];
        let _ = self.peek(&mut header).await.expect("Read called on an empty TcpStream");
        let buf_size: usize = u32::from_le_bytes(header[OP_TYPE_SIZE..OP_TYPE_SIZE+4].try_into()?) as usize;
        // Receive the full message - TODO: Some assertions on the buf_size before creating the vec?
        let mut buf: Vec<u8> = vec![0; buf_size];
        self.read_exact(&mut buf).await?;
        // Create the correct operation
        let operation = try_parse_buf(&buf)?;
        let sender = self.peer_addr()?;
        log(format!("Read a {} from {:?}", operation.to_string(), &sender));
        return Ok((operation, sender.to_string()));
        
    }
}
