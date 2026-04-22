use std::env;

use crate::harbourmaster::Harbourmaster;

mod harbourmaster;
mod harbourmaster_ais;
mod clients_registry;
mod harbourmaster_gps;
mod fms;
mod database_manager;


#[tokio::main]
async fn main() -> () {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    }

    let harbourmaster: Harbourmaster = Harbourmaster::init().await;

    harbourmaster.start().await;

    std::thread::park();
}