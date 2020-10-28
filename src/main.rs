#[macro_use]
extern crate lazy_static;
mod component_manager;
mod constants;
mod easyhash;
mod filesystem;
mod globals;
mod heartbeat;
mod locks;
mod modular;
mod operation;
use async_std;
use std::{env, error, thread, time};
use std::process::exit;
use std::sync::{mpsc};

// Types
pub type BoxedError = Box<dyn error::Error + Send + Sync>;
pub type BoxedErrorResult<T> = std::result::Result<T, BoxedError>;
type ArgResult = (u16);

// Functions
fn main() -> BoxedErrorResult<()> {
    let port = parse_args_or_crash();
    let (operation_sender, operation_receiver) = mpsc::channel();
    async_std::task::block_on(component_manager::startup(port))?;
    component_manager::start_sender(Some(1000), operation_receiver);
    component_manager::start_receiver(Some(1000), operation_sender.clone());
    component_manager::start_maintainer(Some(500), operation_sender.clone());
    component_manager::start_file_server(Some(500), operation_sender.clone());
    component_manager::start_console(None, operation_sender.clone());
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

#[cfg(test)]
mod tests {
use crate::modular::*;
    #[test]
    fn modular_tests() {
        let m1 = Modular::new(1, 7);
        assert_eq!(*m1, 1);
        let m2 = Modular::new(-1, 7);
        assert_eq!(*m2, 6);
        let m3 = Modular::new(1, 7);
        assert_eq!(*(m3 - 2), 6);
        let m4 = Modular::new(6, 7);
        assert_eq!(*(m4 + 2 as i32), 1);
        let m5 = Modular::new(-5435, 1);
        assert_eq!(*m5, 0);
        let m6 = Modular::new(5435, 1);
        assert_eq!(*m6, 0);
    }
}
