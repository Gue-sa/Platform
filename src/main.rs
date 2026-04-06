use std::sync::Arc;

use slint::ToSharedString;

use colored::*;

use crate::{boat::Boat, boat_info::BoatInfo, common::utils::log};

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


slint::include_modules!();


#[tokio::main]
async fn main() {
    let ui = AppWindow::new().expect("REASON");
    let ui_handle = ui.as_weak();

    let boat = Boat::init().await;

    let boat_info: Arc<BoatInfo> = boat.info();

    tokio::spawn(async move {
        boat.start().await;
    });

    ui.on_request_close(move || {
        log("Extinction du système...".yellow());
        slint::quit_event_loop().expect("Erreur lors de la fermeture");
    });

    tokio::spawn(async move {
        let mut interval: tokio::time::Interval = tokio::time::interval(tokio::time::Duration::from_nanos(80_000_000 / 3));

        loop {
            interval.tick().await;
            
            let mmsi: u32 = boat_info.get_static_data().mmsi;
            let name: String = boat_info.get_static_data().name;

            let ui_handle_clone = ui_handle.clone();

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle_clone.upgrade() {
                    ui.set_boat_name(name.to_shared_string());
                    ui.set_boat_mmsi(mmsi as i32);
                }
            });
        }
    });

    let _ = ui.run();
}
