use std::{error, thread, time};
use std::sync::{Arc, Mutex};

type BoxedErrorResult<T> = std::result::Result<T, Box<dyn error::Error>>;
pub type HeartBeatResult = BoxedErrorResult<()>;

const FREQUENCY_DURATION: u64 = 500;
const PORTNUM: u16 = 9000;

lazy_static! {
    pub static ref Count: Mutex<u64> = Mutex::new(0);
    pub static ref FREQUENCY: time::Duration = time::Duration::from_millis(FREQUENCY_DURATION);
}

pub fn run_component(f: &mut dyn Fn() -> HeartBeatResult) {
    loop {
        let fres = f();
        if let Err(e) = fres  {
            println!("Error: {}", e);
        }
        thread::sleep(*FREQUENCY);
    };
}

pub fn sender() -> HeartBeatResult {
    let mut locked_count = Count.lock().unwrap();
    let oldval = *locked_count;
    let newval = oldval + 1;
    *locked_count = newval;
    println!("Updating sendval from {} to {}", oldval, newval);
    Ok(())
}

pub fn receiver() -> HeartBeatResult {
    let val = *Count.lock().unwrap();
    if val % 10 == 0 {
        println!("val = {}", val);
    }
    Ok(())
}
