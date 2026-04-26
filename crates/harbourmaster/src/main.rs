use crate::harbourmaster::Harbourmaster;
use std::env;

mod clients_registry;
mod database_manager;
mod fms;
mod harbourmaster;
mod harbourmaster_ais;
mod harbourmaster_gps;

#[tokio::main]
async fn main() -> () {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    }

    let harbourmaster: Harbourmaster = Harbourmaster::init().await;

    harbourmaster.start().await;

    std::thread::park();
}
