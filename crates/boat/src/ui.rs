use rev_lines::RevLines;
use shared::{boat_info::BoatInfo, config::Config};
use slint::{ModelRc, SharedString, ToSharedString, VecModel, Weak};
use std::{fs::File, io::BufReader, process, rc::Rc, sync::Arc};

// AJOUT DES IMPORTS CROSSTERM
use crossterm::{
    cursor::Show,
    event::DisableMouseCapture,
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use std::io::stdout;

slint::include_modules!();

pub struct Ui {
    ui: AppWindow,
    ui_handle: Weak<AppWindow>,
    boat_info: Arc<BoatInfo>,
}

impl Ui {
    fn get_last_logs_entries(count: usize, logs_filename: &str) -> ModelRc<SharedString> {
        if let Ok(file) = File::open(logs_filename) {
            let rev_lines = RevLines::new(BufReader::new(file));

            let unwraped_logs = VecModel::<SharedString>::default();

            for entry in rev_lines.take(count) {
                if let Ok(line) = entry {
                    unwraped_logs.push(line.into());
                }
            }

            Rc::new(unwraped_logs).into()
        } else {
            Rc::new(VecModel::<SharedString>::default()).into()
        }
    }

    pub fn init(boat_info: Arc<BoatInfo>) -> Self {
        let ui = AppWindow::new().expect("REASON");
        let ui_handle = ui.as_weak();

        Self {
            ui: ui,
            ui_handle: ui_handle,
            boat_info: boat_info,
        }
    }

    pub fn start(&self) -> () {
        let ui_handle_clone = self.ui_handle.clone();
        let boat_info_arc = self.boat_info.clone();

        self.ui.on_close(move || {
            let _ = slint::quit_event_loop();
        });

        tokio::spawn(async move {
            let mut interval: tokio::time::Interval =
                tokio::time::interval(tokio::time::Duration::from_nanos(80_000_000 / 3));

            loop {
                interval.tick().await;

                let static_data = boat_info_arc.get_static_data().unwrap();
                let voyage_data = boat_info_arc.get_voyage_data().unwrap();
                let nav_data = boat_info_arc.get_navigation_data().unwrap();

                let name = static_data.name().to_string();
                let mmsi = *static_data.mmsi();
                let imo = *static_data.imo_number();
                let boat_type = *static_data.type_of_ship_and_cargo_type();
                let lon = *nav_data.longitude();
                let lat = *nav_data.latitude();
                let heading = *nav_data.true_heading();
                let speed = *nav_data.speed_over_ground();
                let turn_rate = *nav_data.rate_of_turn();
                let destination = voyage_data.destination().to_string();
                let eta_month = *voyage_data.eta_month();
                let eta_day = *voyage_data.eta_day();
                let eta_hour = *voyage_data.eta_hour();
                let eta_min = *voyage_data.eta_minute();

                let ui_weak = ui_handle_clone.clone();

                let _ = slint::invoke_from_event_loop(move || {
                    let config = Config::load().unwrap();

                    if let Some(ui) = ui_weak.upgrade() {
                        let boat_data = ui.global::<BoatData>();
                        let system_logs: ModelRc<SharedString> =
                            Ui::get_last_logs_entries(300, config.boat_sys_logs_filename());
                        let ais_logs: ModelRc<SharedString> =
                            Ui::get_last_logs_entries(300, config.boat_ais_logs_filename());
                        let gps_logs: ModelRc<SharedString> =
                            Ui::get_last_logs_entries(300, config.boat_gps_logs_filename());
                        let satcom_logs: ModelRc<SharedString> =
                            Ui::get_last_logs_entries(300, config.boat_satcom_logs_filename());
                        let computer_logs: ModelRc<SharedString> =
                            Ui::get_last_logs_entries(300, config.boat_computer_logs_filename());

                        boat_data.set_boat_name(name.to_shared_string());
                        boat_data.set_boat_mmsi(mmsi as i32);
                        boat_data.set_boat_imo(imo as i32);
                        boat_data.set_boat_type(boat_type as i32);
                        boat_data.set_boat_longitude(lon as i32);
                        boat_data.set_boat_latitude(lat as i32);
                        boat_data.set_boat_heading(heading.into());
                        boat_data.set_boat_speed(speed.into());
                        boat_data.set_boat_turn_rate(turn_rate.into());
                        boat_data.set_boat_destination(destination.to_shared_string());
                        boat_data.set_boat_eta_month(eta_month.into());
                        boat_data.set_boat_eta_day(eta_day.into());
                        boat_data.set_boat_eta_hour(eta_hour.into());
                        boat_data.set_boat_eta_minute(eta_min.into());

                        boat_data.set_sys_log_msgs(system_logs);
                        boat_data.set_ais_log_msgs(ais_logs);
                        boat_data.set_gps_log_msgs(gps_logs);
                        boat_data.set_satcom_log_msgs(satcom_logs);
                        boat_data.set_computer_log_msgs(computer_logs);
                    }
                });
            }
        });

        let _ = self.ui.run();

        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture, Show);

        process::exit(0);
    }
}
