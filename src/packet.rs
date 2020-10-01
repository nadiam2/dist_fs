use bincode;
use crate::BoxedErrorResult;
use crate::component_manager::PacketSender;
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

// Types
type BoxedPacket = Box<dyn PacketWriteExecute + Send + Sync>;

// Packet Queue Item
pub struct PacketQueueItem {
    pub dests: Vec<&'static str>,
    pub packet: Packet
}

impl PacketQueueItem {
    pub fn write_all(&self, socket: &UdpSocket) -> BoxedErrorResult<()> {
        let serialized = self.packet.to_bytes()?;
        for dest in &self.dests {
            socket.send_to(&serialized, &dest)?;
        }
        Ok(())
    }
}

// Traits
pub trait PacketWriteExecute {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>>;
    fn execute(&self, source: String, sender: &PacketSender) -> BoxedErrorResult<()>; // TODO: Change the () to a useful type for forwarding requests and such
}

// Packet Structs
#[derive(Serialize, Deserialize)]
pub struct JoinPacket {}

#[derive(Serialize, Deserialize)]
pub struct MembershipListPacket {
    list: Vec<String>
}

// Enums
pub enum Packet {
    Join(BoxedPacket),
    MembershipList(BoxedPacket)
}

// Trait Impls
impl PacketWriteExecute for JoinPacket {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['J' as u8]))
    }
    fn execute(&self, source: String, sender: &PacketSender) -> BoxedErrorResult<()> {
        println!("Would have called the join rpc for jpacket");
        Ok(())
    }
}

impl PacketWriteExecute for MembershipListPacket {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, vec!['M' as u8]))
    }
    fn execute(&self, source: String, sender: &PacketSender) -> BoxedErrorResult<()> {
        println!("Would have called the memblist rpc for mpacket on {:?}", self.list);
        Ok(())
    }
}

impl PacketWriteExecute for Packet {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        let serialized = match self {
            Packet::Join(packet) |
            Packet::MembershipList(packet) => packet.to_bytes()
        }?;
        Ok(serialized)
    }
    fn execute(&self, source: String, sender: &PacketSender) -> BoxedErrorResult<()> {
        match self {
            Packet::Join(packet) |
            Packet::MembershipList(packet) => packet.execute(source, sender)
        };
        Ok(())
    }
}

// Impls
impl Packet {
    pub fn read(socket: &UdpSocket) -> BoxedErrorResult<(Self, String)> { // TODO: Make this return a result with a tuple containing the source address
        // Parse the header
        let mut header: Vec<u8> = vec![0; 5];
        let _ = socket.peek_from(&mut header).expect("Read called on an empty socket.");
        let buf_size: u32 = u32::from_le_bytes(header[1..5].try_into().unwrap());
        // Receive the full message
        let mut buf: Vec<u8> = vec![0; buf_size as usize];
        let (_, sender) = socket.recv_from(&mut buf)
            .expect("Read called on an empty socket.");
        // Create the correct packet
        let packet = match buf[0] as char {
            'J' => Packet::Join(Box::new(
                bincode::deserialize::<JoinPacket>(&buf[5..]).unwrap())),
            'M' => Packet::MembershipList(Box::new(
                bincode::deserialize::<MembershipListPacket>(&buf[5..]).unwrap())),
            _   => return Err(String::from("Read unrecognized packet header").into())
        };
        return Ok((packet, sender.to_string()));
    }
}

// Functions
fn create_buf<T>(obj: &T, mut base: Vec<u8>) -> Vec<u8>
where T: Serialize {
    let serialized = bincode::serialize(obj).unwrap();
    let size: u32 = (5 + serialized.len()) as u32;
    base.extend_from_slice(&size.to_le_bytes());
    base.extend_from_slice(&serialized);
    return base;
}
