use crate::BoxedErrorResult;
use crate::globals;
use crate::heartbeat;
use crate::operation::*;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::net::{Ipv4Addr, UdpSocket};
use std::str::FromStr;
use std::sync::{mpsc};
use std::{thread, time};

// Types
type FrequencyInterval = Option<u64>;
pub type ComponentResult = BoxedErrorResult<()>;
pub type OperationSender = mpsc::Sender<SendableOperation>;
pub type OperationReceiver = mpsc::Receiver<SendableOperation>;

// Component Starters
pub fn start_sender(freq_interval: FrequencyInterval, receiver: OperationReceiver) {
    thread::spawn(move || {
        start_component(&mut sender, receiver, freq_interval);
    });
}

pub fn start_receiver(freq_interval: FrequencyInterval, sender: OperationSender) {
    thread::spawn(move || {
        start_component(&mut receiver, sender, freq_interval);
    });    
}

pub fn start_maintainer(freq_interval: FrequencyInterval, sender: OperationSender) {
    thread::spawn(move || {
        start_component(&mut heartbeat::maintainer, sender, freq_interval);
    });    
}

pub fn start_console(freq_interval: FrequencyInterval, sender: OperationSender) {
    thread::spawn(move || {
        start_component(&mut console, sender, freq_interval);
    });    

}

// Utility Functions
pub fn startup(port: u16) -> BoxedErrorResult<()> {
    startup_log_file(port);
    let udp_socket_addr = get_udp_scket_addr(port)?;
    globals::UDP_SOCKET.write(UdpSocket::bind(&udp_socket_addr)?);
    globals::IS_JOINED.write(false);
    globals::MEMBERSHIP_LIST.write(Vec::new());
    globals::SUCCESSOR_LIST.write(Vec::new());
    globals::PREDECESSOR_LIST.write(Vec::new());
    globals::PREDECESSOR_TIMESTAMPS.write(HashMap::new());
    globals::MY_IP_ADDR.write(udp_socket_addr.to_string());
    globals::DEBUG.write(true);
    Ok(())
}

fn startup_log_file(port: u16) -> BoxedErrorResult<()> {
    if let Err(_) = fs::create_dir("logs") {}
    let timestamp = heartbeat::get_timestamp()?;
    let debug_file = format!("logs/port_{}_{:020}.txt", port, timestamp);
    globals::LOG_FILE.write(OpenOptions::new()
                              .read(true)
                              .write(true)
                              .create(true)
                              .open(debug_file)?);
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
    // Separate ifs because the if let still experimental with another expression
    if *globals::DEBUG.read() {
        if let Err(e) = fres {
            // TODO: Add some better error handling
            println!("Error: {}", e);
        }
    }
}

// Components
pub fn sender(receiver: &OperationReceiver) -> ComponentResult {
    // Empty the queue by sending all outstanding operations
    // Do this before heartbeating so that we can empty after a leave
    let udp_socket = globals::UDP_SOCKET.read();
    while let Ok(queue_item) = receiver.try_recv() {
        queue_item.write_all(&udp_socket)?;
    }

    // Send heartbeat packets    
    if is_joined() {
        heartbeat::send_heartbeats();
    }
    Ok(())
}

pub fn receiver(sender: &OperationSender) -> ComponentResult {
    let udp_socket = globals::UDP_SOCKET.read();
    loop {
        if is_joined() {
            let (operation, source) = read_operation(&*udp_socket)?;
            operation.execute(source, &sender)?;
        } else {
            // Drop the packet
            udp_socket.recv_from(&mut vec![0])?;
        }
    }
}

pub fn console(sender: &OperationSender) -> ComponentResult {
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

// TODO: Maybe find another place for this - Also: borrow or owned?
pub fn log(msg: String) -> BoxedErrorResult<()> {
    writeln!(*globals::LOG_FILE.get_mut(), "{}", msg)?;
    Ok(())
}
