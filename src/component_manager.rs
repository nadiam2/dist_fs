use crate::BoxedErrorResult;
use crate::globals;
use crate::heartbeat;
use crate::locks::*;
use crate::packet::*;
use std::convert::TryFrom;
use std::net::{Ipv4Addr, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::{io, thread, time};

// Types
type FrequencyInterval = Option<u64>;
type ComponentResult = BoxedErrorResult<()>;
pub type PacketSender = mpsc::Sender<PacketQueueItem>;
pub type PacketReceiver = mpsc::Receiver<PacketQueueItem>;

// Component Starters
pub fn start_sender(freq_interval: FrequencyInterval, receiver: PacketReceiver) {
    thread::spawn(move || {
        start_component(&mut sender, receiver, freq_interval);
    });
}

pub fn start_receiver(freq_interval: FrequencyInterval, sender: PacketSender) {
    thread::spawn(move || {
        start_component(&mut receiver, sender, freq_interval);
    });    
}

pub fn start_console(freq_interval: FrequencyInterval, sender: PacketSender) {
    thread::spawn(move || {
        start_component(&mut console, sender, freq_interval);
    });    

}

// Utility Functions
pub fn startup(port: u16) -> BoxedErrorResult<()> {
    // globals::UDP_SOCKET.write(UdpSocket::bind(format!("localhost:{}", port))?);
    let udp_socket_addr = get_udp_scket_addr(port)?;
    globals::UDP_SOCKET.write(UdpSocket::bind(&udp_socket_addr)?);
    globals::IS_JOINED.write(false);
    globals::MEMBERSHIP_LIST.write(Vec::new());
    globals::MY_IP_ADDR.write(udp_socket_addr.to_string()); //  TODO: This must change for external hosts to work
    Ok(())
}

fn start_component<T, A>(f: &mut dyn Fn(&A) -> BoxedErrorResult<T>, arg: A, freq_interval: FrequencyInterval) {
    let freq = parse_frequency(freq_interval);
    loop {
        run_component(f, &arg);
        thread::sleep(time::Duration::from_millis(freq));
    }
}

fn run_component<T, A>(f: &mut dyn Fn(&A) -> BoxedErrorResult<T>, arg: &A) {
    let fres = f(arg);
    if let Err(e) = fres  {
        // TODO: Add some better error handling
        println!("Error: {}", e);
    }
}

// Components
pub fn sender(receiver: &PacketReceiver) -> ComponentResult {
    if !is_joined() { return Ok(()) }

    let udp_socket = globals::UDP_SOCKET.read();
    // Send heartbeat packets
    
    
    // Empty the queue by sending all remaining packets
    while let Ok(queue_item) = receiver.try_recv() {
        println!("going to send to {:?}", queue_item.dests
        );
        queue_item.write_all(&udp_socket);
    }
    Ok(())
}

pub fn receiver(sender: &PacketSender) -> ComponentResult {
    let udp_socket = globals::UDP_SOCKET.read();
    loop {
        if is_joined() {
            let (packet_op, source) = read_packet(&*udp_socket)?;
            packet_op.execute(source, &sender);
        } else {
            // Drop the packet
            udp_socket.recv_from(&mut vec![0]);
        }
    }
}

pub fn console(sender: &PacketSender) -> ComponentResult {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    match line.trim() {
        "join"  => heartbeat::join(sender)?,
        "leave" => heartbeat::leave(sender)?,
        "print" => heartbeat::print()?,
        _       => println!("Invalid command. (Maybe replace with a help func)")
    }
    Ok(())
}

// Helper Functions
pub fn is_joined() -> bool {
    *globals::IS_JOINED.read()
}

fn parse_frequency(freq_interval: FrequencyInterval) -> u64 {
    match freq_interval {
        Some(interval) => interval,
        None           => 0
    }
}

// TODO: Eventually, change this to external IPs
fn get_udp_scket_addr(port: u16) -> BoxedErrorResult<String> {
    let local_addr = get_local_addr()?;
    Ok(format!("{}:{}", local_addr, port))
}

fn get_local_addr() -> BoxedErrorResult<String> {
    for iface in get_if_addrs::get_if_addrs().unwrap() {
        if let get_if_addrs::IfAddr::V4(v4_addr) = iface.addr {
            if v4_addr.netmask == Ipv4Addr::from_str("255.255.255.0").unwrap() {
                return Ok(v4_addr.ip.to_string());
            }
        }
    }
    Err("Could not find valid local IPv4 address".to_string().into())
}
