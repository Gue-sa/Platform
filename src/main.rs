use crate::boat::Boat;


mod common;
mod ais;
mod boat;
mod slot;
mod slots_map;
mod gps;
mod voyage;
mod ui;
mod systemstate;
mod shared;
mod satcom;


#[tokio::main]
async fn main() {
    let boat: Boat = Boat::init().await;

    boat.start();
}
