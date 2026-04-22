use std::env;

use crate::boat::Boat;

mod boat;
mod boat_ais;
mod boat_gps;
mod common;
mod systemstate;
mod ui;
mod voyage;
mod board_computer;

#[tokio::main]
async fn main() {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    }
    
    let boat: Boat = Boat::init().await;

    boat.start().await;
}
