mod heartbeat;

use heartbeat::*;
use std::{env, thread, time};

fn start_sender() {
    thread::spawn(move || {
        spawn_heartbeater_component(&mut sender);
    });
}

fn start_receiver() {
    thread::spawn(move || {
        spawn_heartbeater_component(&mut receiver);
    });    
}

fn main() {
    start_sender();
    loop {
        thread::sleep(time::Duration::from_millis(1000));
    }
}
