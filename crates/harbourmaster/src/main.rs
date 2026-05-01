use crate::harbourmaster::Harbourmaster;
use shared::config::Config;
use std::env;

mod database_manager;
mod fms;
mod harbourmaster;
mod harbourmaster_ais;
mod harbourmaster_gps;
mod harbourmaster_web_ui;

#[tokio::main]
async fn main() -> () {
    if Config::load().is_some() {
        unsafe {
            env::set_var("RUST_BACKTRACE", "1");
        }

        let harbourmaster: Harbourmaster = Harbourmaster::init()
            .await
            .expect("L'initialisation de la capitainerie a échoué");

        harbourmaster.start().await;

        std::thread::park();
    } else {
        println!(
            "Fichier de configuration non trouvé. Veuillez lancer le programme depuis le laucher."
        )
    }
}
