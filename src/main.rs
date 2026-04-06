use crate::boat::Boat;

mod common;
mod ais;
mod antenna;
mod boat;
mod message;
mod slot;
mod slots_map;
mod boat_info;
mod boats_registry;
mod gps;
mod display;


#[tokio::main]
async fn main() {
    let boat: Boat = Boat::init().await;
    
    boat.start().await;
}
