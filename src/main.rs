use std::sync::Arc;

use crate::harbourmaster::Harbourmaster;


mod common;
mod gps;
mod harbourmaster;
mod station;
mod clients_registry;
mod antenna;


fn main() {
    let harbourmaster = Harbourmaster::init();
    Arc::new(harbourmaster).start();

    loop {
        
    }
}
