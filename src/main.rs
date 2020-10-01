#[macro_use]
extern crate lazy_static;
mod component_manager;
mod constants;
mod heartbeat;
mod locks;
mod packet;
use std::{env, error, thread, time};
use std::process::exit;
use std::sync::{mpsc};

// Types
pub type BoxedErrorResult<T> = std::result::Result<T, Box<dyn error::Error>>;
type ArgResult = (u16);

// Functions
fn main() -> BoxedErrorResult<()> {
    let port = parse_args_or_crash();
    let (packet_sender, packet_receiver) = mpsc::channel();
    component_manager::startup(port)?;
    component_manager::start_sender(Some(1000), packet_receiver);
    component_manager::start_receiver(Some(1000), packet_sender.clone());
    component_manager::start_console(None, packet_sender.clone());
    loop {
        thread::sleep(time::Duration::from_millis(1000));
    }
}

fn parse_args_or_crash() -> ArgResult {
    match try_parse_args() {
        Ok(args) => args,
        Err(e)   => {
            println!("Error parsing arguments: {}", e);
            help();
            exit(1);
        }
    }
}

fn try_parse_args() -> BoxedErrorResult<ArgResult> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => {
            let port: u16 = args[1].parse()?;
            Ok(port)
        },
        _ => Err(String::from("Incorrect number of arguments").into())
    }
}

fn help() {
    println!("Usage: ./BIN PORT_NUM");
}
