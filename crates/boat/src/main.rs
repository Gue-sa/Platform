use std::{env, thread};

use crate::{boat::Boat, serial_driver::SerialDriver};

mod boat;
mod boat_ais;
mod boat_gps;
mod common;
mod systemstate;
mod ui;
mod voyage;
mod board_computer;
mod serial_driver;

#[tokio::main]
async fn main() {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    }
    
    //let boat: Boat = Boat::init().await;

    //boat.start().await;

    let serialdriver = SerialDriver::init(/*boat.serial_rx, boat.serial_tx*/);

    serialdriver.start().await;

    thread::park();
}
