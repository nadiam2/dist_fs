use bincode;
use crate::BoxedErrorResult;
use crate::component_manager::PacketSender;
use crate::globals;
use crate::heartbeat;
use serde::{Serialize, Deserialize};
use std::convert::TryInto;
use std::error;
use std::net::UdpSocket;
use std::fmt;


// OPCODES
// 'J': Join -> empty packet to introducer
// 'L': Leave -> data should contain node id of node leaving
// 'M': Membership List -> data should be list itself
//      and have listener update their list (usually will be initializing it)
// 'N': New Node -> data should contain id of new node

// Constants
static HEADER_SIZE: usize = 5;

// Types
type BoxedPacket = Box<dyn PacketWriteExecute + Send + Sync>;

// Packet Queue Item
pub struct PacketQueueItem {
    pub dests: Vec<String>,
    pub packet: BoxedPacket
}

impl PacketQueueItem {
    pub fn write_all(&self, socket: &UdpSocket) -> BoxedErrorResult<()> {
        let serialized = self.packet.to_bytes()?;
        for dest in &self.dests {
            socket.send_to(&serialized, &dest)?;
        }
        Ok(())
    }
    pub fn for_list(dest_ids: Vec<String>, packet: BoxedPacket) -> Self {
        PacketQueueItem{
            dests: heartbeat::ips_from_ids(dest_ids),
            packet: packet
        }
    }
    pub fn for_everyone(packet: BoxedPacket) -> Self {
        Self::for_list((*globals::MEMBERSHIP_LIST.read()).clone(), packet)
    }
    pub fn for_single(dest_id: String, packet: BoxedPacket) -> Self {
        Self::for_list(vec![dest_id], packet)
    }
}

// Traits
pub trait PacketWriteExecute {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>>;
    fn execute(&self, source: String, sender: &PacketSender) -> BoxedErrorResult<()>;
}

// Packet Structs
#[derive(Serialize, Deserialize, Debug)]
pub struct JoinPacket {
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewMemberPacket {
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MembershipListPacket {
    membership_list: Vec<String>
}

// Trait Impls
impl PacketWriteExecute for JoinPacket {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['J' as u8]))
    }
    fn execute(&self, source: String, sender: &PacketSender) -> BoxedErrorResult<()> {
        // Add the new guy and send it to everyone
        heartbeat::insert_node(&self.id)?;
        sender.send(PacketQueueItem::for_everyone(Box::new(NewMemberPacket{
            id: self.id.clone()
        })))?;
        sender.send(PacketQueueItem::for_single(self.id.to_string(), Box::new(MembershipListPacket{
            membership_list: globals::MEMBERSHIP_LIST.read().clone()
        })))?;
        Ok(())
    }
}

impl PacketWriteExecute for NewMemberPacket {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['N' as u8]))
    }
    fn execute(&self, source: String, _sender: &PacketSender) -> BoxedErrorResult<()> {
        heartbeat::insert_node(&self.id);
        // MAYBE TODO: Do we need more redundancy to make sure joins are not missed?
        Ok(())
    }
}

impl PacketWriteExecute for MembershipListPacket {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['M' as u8]))
    }
    fn execute(&self, source: String, sender: &PacketSender) -> BoxedErrorResult<()> {
        heartbeat::merge_membership_list(&self.membership_list);
        Ok(())
    }
}

// Functions
pub fn read_packet(socket: &UdpSocket) -> BoxedErrorResult<(BoxedPacket, String)> {
    // Parse the header
    let mut header: Vec<u8> = vec![0; HEADER_SIZE];
    let _ = socket.peek_from(&mut header).expect("Read called on an empty socket.");
    let buf_size: u32 = u32::from_le_bytes(header[1..5].try_into().unwrap());
    // Receive the full message
    let mut buf: Vec<u8> = vec![0; buf_size as usize];
    let (_, sender) = socket.recv_from(&mut buf)
        .expect("Read called on an empty socket.");
    // Create the correct packet
    let packet: BoxedPacket = match buf[0] as char {
        'J' => Box::new(bincode::deserialize::<JoinPacket>(&buf[HEADER_SIZE..]).unwrap()),
        'N' => Box::new(bincode::deserialize::<NewMemberPacket>(&buf[HEADER_SIZE..]).unwrap()),
        'M' => Box::new(bincode::deserialize::<MembershipListPacket>(&buf[HEADER_SIZE..]).unwrap()),
        _   => return Err(String::from("Read unrecognized packet header").into())
    };
    return Ok((packet, sender.to_string()));
}

fn create_buf<T>(obj: &T, mut base: Vec<u8>) -> Vec<u8>
where T: Serialize {
    let serialized = bincode::serialize(obj).unwrap();
    let size: u32 = (5 + serialized.len()) as u32;
    base.extend_from_slice(&size.to_le_bytes());
    base.extend_from_slice(&serialized);
    return base;
}
