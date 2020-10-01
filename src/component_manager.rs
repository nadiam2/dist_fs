use crate::BoxedErrorResult;
use crate::heartbeat;
use crate::packet::*;
use std::convert::TryFrom;
use std::net::UdpSocket;
use std::ops::Deref;
use std::sync::{Arc, mpsc, Mutex, MutexGuard, RwLock, RwLockReadGuard};
use std::{io, thread, time};

// Types
type FrequencyInterval = Option<u64>;
type ComponentResult = BoxedErrorResult<()>;
pub type PacketSender = mpsc::Sender<PacketQueueItem>;
pub type PacketReceiver = mpsc::Receiver<PacketQueueItem>;

// Vars
lazy_static! {
    pub static ref UDP_SOCKET_LOCK: RwLockOption<UdpSocket> = RwLockOption::new();
    pub static ref IS_JOINED_LOCK: RwLockOption<bool> = RwLockOption::new();
}

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
    UDP_SOCKET_LOCK.write(UdpSocket::bind(format!("localhost:{}", port))?);
    IS_JOINED_LOCK.write(false);
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

    let udp_socket = UDP_SOCKET_LOCK.read();
    loop {
        let queue_item = receiver.recv()?;
        queue_item.write_all(&udp_socket);
    }
}

pub fn receiver(sender: &PacketSender) -> ComponentResult {
    let udp_socket = UDP_SOCKET_LOCK.read();
    loop {
        if is_joined() {
            let (packet_op, source) = Packet::read(&*udp_socket)?;
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

// Lock Shenanigans
pub struct RwLockOption<T> {
    lock: RwLock<Option<T>>
}

impl<T> RwLockOption<T> {
    pub fn new() -> Self {
        RwLockOption{lock: RwLock::new(None)}
    }
    pub fn read<'a>(&'a self) -> InnerRwLock<'a, T> {
        let guard = self.lock.read().unwrap();
        InnerRwLock{guard}
    }
    pub fn write(&self, val: T) {
        let mut opt = self.lock.write().unwrap();
        *opt = Some(val);
    }
}

pub struct InnerRwLock<'a, T> {
    guard: RwLockReadGuard<'a, Option<T>>
}

impl<'a, T> Deref for InnerRwLock<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

pub struct MutexOption<T> {
    lock: Mutex<Option<T>>
}

impl<T> MutexOption<T> {
    pub fn new() -> Self {
        MutexOption{lock: Mutex::new(None)}
    }
    pub fn read<'a>(&'a self) -> InnerMutex<'a, T> {
        let guard = self.lock.lock().unwrap();
        InnerMutex{guard}
    }
    pub fn write(&self, val: T) {
        let mut opt = self.lock.lock().unwrap();
        *opt = Some(val);
    }
}

pub struct InnerMutex<'a, T> {
    guard: MutexGuard<'a, Option<T>>
}

impl<'a, T> Deref for InnerMutex<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

// Helper Functions
pub fn is_joined() -> bool {
    *IS_JOINED_LOCK.read()
}

fn parse_frequency(freq_interval: FrequencyInterval) -> u64 {
    match freq_interval {
        Some(interval) => interval,
        None           => 0
    }
}
