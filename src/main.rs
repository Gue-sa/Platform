use crate::harbourmaster::Harbourmaster;

mod common;
mod harbourmaster;
mod harbourmaster_ais;
mod antenna;
mod clients_registry;
mod gps;
mod station;
mod shared;


#[tokio::main]
async fn main() -> () {
    let harbourmaster: Harbourmaster = Harbourmaster::init().await;

    harbourmaster.start().await;

    std::thread::park();
}