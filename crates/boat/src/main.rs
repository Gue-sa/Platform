use shared::config::Config;

use crate::{boat::Boat, serial_driver::SerialDriver};
use std::thread;

mod board_computer;
mod boat;
mod boat_ais;
mod boat_gps;
mod common;
mod serial_driver;
mod systemstate;
mod ui;
mod voyage;

#[tokio::main]
async fn main() {
    if Config::load().is_some() {
        let boat = Boat::init()
            .await
            .expect("L'initialisation du bateau a échoué");

        boat.start().await;

        //let serialdriver = SerialDriver::init(/*boat.serial_rx, boat.serial_tx*/);

        //serialdriver.start().await;

        thread::park();
    } else {
        println!(
            "Fichier de configuration non trouvé. Veuillez lancer le programme depuis le launcher."
        )
    }
}
