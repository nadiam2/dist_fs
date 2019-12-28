extern crate rand;

use rand::Rng;
use std::{error, thread, time};

const FREQUENCY: u64 = 500;

type BoxedErrorResult<T> = std::result::Result<T, Box<dyn error::Error>>;
pub type HeartBeatResult = BoxedErrorResult<()>;

pub struct HeartBeatInfo {
    pub portnum: u16,
    pub frequency: time::Duration,
    pub send_count: u32,
}

impl HeartBeatInfo {
    fn new() -> HeartBeatInfo {
        let mut rng = rand::thread_rng();
        HeartBeatInfo {
            portnum: rng.gen(),
            frequency: time::Duration::from_millis(FREQUENCY),
            send_count: 0
        }
    }
}

pub fn spawn_heartbeater_component(f: &mut dyn FnMut(&mut HeartBeatInfo) -> HeartBeatResult) {
    let mut hbinfo = HeartBeatInfo::new();
    loop {
        let fres = f(&mut hbinfo);
        if let Err(e) = fres  {
            println!("Error: {}", e);
        }
        thread::sleep(hbinfo.frequency);
    };
}

pub fn sender(hbinfo: &mut HeartBeatInfo) -> HeartBeatResult {
    let oldval = hbinfo.send_count;
    let newval = oldval + 1;
    hbinfo.send_count = newval;
    println!("Updating sendval from {} to {}", oldval, newval);
    Ok(())
}

pub fn receiver(hbinfo: &mut HeartBeatInfo) -> HeartBeatResult {
    let oldval = hbinfo.send_count;
    let newval = oldval + 1;
    hbinfo.send_count = newval;
    println!("Updating sendval from {} to {}", oldval, newval);
    Ok(())
}
