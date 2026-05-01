use shared::config::Config;

use crate::{boat::Boat, boat_logs_cli::BoatLogsCli, serial_driver::SerialDriver};
use std::{env, thread};

mod board_computer;
mod boat;
mod boat_ais;
mod boat_gps;
mod boat_logs_cli;
mod common;
mod serial_driver;
mod systemstate;
mod ui;
mod voyage;

#[tokio::main]
async fn main() {
    if Config::load().is_some() {
        unsafe {
            env::set_var("RUST_BACKTRACE", "1");
        }

        let boat: Boat = Boat::init()
            .await
            .expect("L'initialisation du bateau a échoué");

        boat.start().await;

        //let serialdriver = SerialDriver::init(/*boat.serial_rx, boat.serial_tx*/);

        //serialdriver.start().await;

        thread::park();
    } else {
        println!(
            "Fichier de configuration non trouvé. Veuillez lancer le programme depuis le laucher."
        )
    }
}
