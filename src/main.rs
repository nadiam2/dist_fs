#[macro_use]
extern crate lazy_static;

mod constants;
mod heartbeat;

use std::{env, thread, time};

fn start_sender() {
    thread::spawn(move || {
        heartbeat::run_component(&mut heartbeat::sender);
    });
}

fn start_receiver() {
    thread::spawn(move || {
        heartbeat::run_component(&mut heartbeat::receiver);
    });    
}

fn main() {
    start_sender();
    start_receiver();
    loop {
        thread::sleep(time::Duration::from_millis(1000));
    }
}
