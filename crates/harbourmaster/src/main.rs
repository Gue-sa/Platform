use crate::harbourmaster::Harbourmaster;

mod harbourmaster;
mod harbourmaster_ais;
mod clients_registry;
mod harbourmaster_gps;
mod fms;
mod database_manager;


#[tokio::main]
async fn main() -> () {
    let harbourmaster: Harbourmaster = Harbourmaster::init().await;

    harbourmaster.start().await;

    std::thread::park();
}