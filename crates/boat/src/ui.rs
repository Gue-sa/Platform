use std::{fs::File, io::BufReader, rc::Rc, sync::Arc};

use colored::*;
use rev_lines::RevLines;
use shared::boat_info::{BoatInfo, NavigationData, StaticData, VoyageData};
use slint::{ModelRc, SharedString, ToSharedString, VecModel, Weak};

use crate::common::utils::log;

slint::include_modules!();

pub struct Ui {
    ui: AppWindow,
    ui_handle: Weak<AppWindow>,
    boat_info: Arc<BoatInfo>,
}

impl Ui {
    fn get_last_logs_entries(count: usize) -> ModelRc<SharedString> {
        let file: File = File::open("logs.log").unwrap();
        let rev_lines: RevLines<BufReader<File>> = RevLines::new(BufReader::new(file));

        let unwraped_logs: VecModel<SharedString> = VecModel::<SharedString>::default();

        for entry in rev_lines.take(count) {
            if let Ok(line) = entry {
                unwraped_logs.push(line.into());
            }
        }

        Rc::new(unwraped_logs).clone().into()
    }

    pub fn init(boat_info: Arc<BoatInfo>) -> Self {
        let ui: AppWindow = AppWindow::new().expect("REASON");
        let ui_handle: slint::Weak<AppWindow> = ui.as_weak();

        Self {
            ui: ui,
            ui_handle: ui_handle,
            boat_info: boat_info,
        }
    }

    pub fn start(&self) -> () {
        let ui_handle_clone: slint::Weak<AppWindow> = self.ui_handle.clone();
        let boat_info_clone: Arc<BoatInfo> = self.boat_info.clone();

        self.ui.on_close(move || {
            log("Extinction du système...".yellow());
            slint::quit_event_loop().expect("Erreur lors de la fermeture");
        });

        tokio::spawn(async move {
            let mut interval: tokio::time::Interval =
                tokio::time::interval(tokio::time::Duration::from_nanos(80_000_000 / 3));

            loop {
                interval.tick().await;

                let static_data: StaticData = boat_info_clone.get_static_data();
                let voyage_data: VoyageData = boat_info_clone.get_voyage_data();
                let nav_data: NavigationData = boat_info_clone.get_navigation_data();

                let name: String = static_data.name().to_string();
                let mmsi: u32 = *static_data.mmsi();
                let imo: u32 = *static_data.imo_number();
                let boat_type: u8 = *static_data.type_of_ship_and_cargo_type();
                let longitude: u32 = *nav_data.longitude();
                let latitude: u32 = *nav_data.latitude();
                let heading: u16 = *nav_data.true_heading();
                let speed: u16 = *nav_data.speed_over_ground();
                let turn_rate: i8 = *nav_data.rate_of_turn();
                let destination: String = voyage_data.destination().to_string();
                let eta_month: u8 = *voyage_data.eta_month();
                let eta_day: u8 = *voyage_data.eta_day();
                let eta_hour: u8 = *voyage_data.eta_hour();
                let eta_minute: u8 = *voyage_data.eta_minute();

                let ui_weak: Weak<AppWindow> = ui_handle_clone.clone();

                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        let boat_data = ui.global::<BoatData>();
                        let logs: ModelRc<SharedString> = Ui::get_last_logs_entries(300);

                        boat_data.set_boat_name(name.to_shared_string());
                        boat_data.set_boat_mmsi(mmsi as i32);
                        boat_data.set_boat_imo(imo as i32);
                        boat_data.set_boat_type(boat_type as i32);
                        boat_data.set_boat_longitude(longitude as i32);
                        boat_data.set_boat_latitude(latitude as i32);
                        boat_data.set_boat_heading(heading as i32);
                        boat_data.set_boat_speed(speed as i32);
                        boat_data.set_boat_turn_rate(turn_rate as i32);
                        boat_data.set_boat_destination(destination.to_shared_string());
                        boat_data.set_boat_eta_month(eta_month as i32);
                        boat_data.set_boat_eta_day(eta_day as i32);
                        boat_data.set_boat_eta_hour(eta_hour as i32);
                        boat_data.set_boat_eta_minute(eta_minute as i32);

                        boat_data.set_log_msgs(logs);
                    }
                });
            }
        });

        self.ui.run();
    }
}
