use crate::boat::Boat;

mod boat;
mod boat_ais;
mod boat_antenna;
mod boat_gps;
mod common;
mod satcom;
mod shared;
mod slot;
mod slots_map;
mod systemstate;
mod ui;
mod voyage;
mod board_computer;

#[tokio::main]
async fn main() {
    let boat: Boat = Boat::init().await;

    boat.start().await;
}
