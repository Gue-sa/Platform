use std::{
    sync::{Arc, Mutex, mpsc::Sender},
    time::Duration,
};

use colored::Colorize;
use shared::{boat_info::BoatInfo, common::types::LogEvent};
use tokio::{sync::Notify, time::sleep};

use crate::{
    common::constants::{
        ANGLE_UNCERTAINTY_RADIUS_MEDIUM, NAV_PARAMS_CHECK_DELAY,
        POSITION_UNCERTAINTY_RADIUS_MEDIUM, ROUTE_CHECK_DELAY,
    },
    serial_driver::SerialDriver,
    voyage::{Voyage, VoyageSegment},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
enum RotationDirection {
    Clockwise,
    Anticlockwise,
    Undefined,
}

#[derive(Clone)]
struct NavigatorState {
    positions_history: Vec<(u16, u16)>,
    heading_estimate: u16,
    speed_estimate: u8,
    is_correcting_course: bool,
    nav_params_check_pulse: Arc<Notify>,
    route_check_pulse: Arc<Notify>,
    obstacles_check_pulse: Arc<Notify>,
}

#[derive(Clone)]
pub struct Navigator {
    voyage: Arc<Mutex<Option<Voyage>>>,
    serial_driver: Arc<Mutex<SerialDriver>>,
    boat_info: Arc<BoatInfo>,
    logs_cli: Sender<LogEvent>,
    state: NavigatorState,
}

impl NavigatorState {
    pub fn new() -> Self {
        Self {
            positions_history: Vec::new(),
            heading_estimate: 0,
            speed_estimate: 0,
            is_correcting_course: false,
            nav_params_check_pulse: Arc::new(Notify::new()),
            route_check_pulse: Arc::new(Notify::new()),
            obstacles_check_pulse: Arc::new(Notify::new()),
        }
    }

    fn calculate_heading_estimate(&self) -> u16 {
        if self.positions_history.len() < 2 {
            return self.heading_estimate;
        }

        let mut estimated_heading = 0.0;

        for window in self.positions_history.windows(2) {
            let (lon1, lat1) = window[0];
            let (lon2, lat2) = window[1];

            let dx = lon2 as f64 - lon1 as f64;
            let dy = lat2 as f64 - lat1 as f64;
            let window_heading = dx.atan2(-dy).to_degrees();

            let normalized = (window_heading + 360.0) % 360.0;
            estimated_heading = (normalized + estimated_heading) / 2.0;
        }

        estimated_heading.round() as u16
    }

    fn calculate_speed_estimate(&self) -> u8 {
        if self.positions_history.len() < 2 {
            return self.speed_estimate;
        }

        let mut estimated_speed = 0.0;

        for window in self.positions_history.windows(2) {
            let (lon1, lat1) = window[0]; // Correction (lon, lat)
            let (lon2, lat2) = window[1];

            let distance = (((lon2 as f64 - lon1 as f64).powf(2.)
                + ((lat2 as f64 - lat1 as f64).powf(2.)))
            .sqrt());

            estimated_speed = (distance / 0.001 + estimated_speed) / 2.0;
        }

        estimated_speed.round() as u8
    }

    pub fn update(&mut self, new_pos: (u16, u16)) {
        self.positions_history.push(new_pos);

        if self.positions_history.len() > 5 {
            self.positions_history.remove(0);
        }

        self.heading_estimate = self.calculate_heading_estimate();
        self.speed_estimate = self.calculate_speed_estimate();
    }
}

impl Navigator {
    pub fn init(
        voyage_opt: Arc<Mutex<Option<Voyage>>>,
        serial_driver: Arc<Mutex<SerialDriver>>,
        boat_info: Arc<BoatInfo>,
        logs_cli: Sender<LogEvent>,
    ) -> Self {
        Self {
            voyage: voyage_opt,
            serial_driver: serial_driver,
            boat_info: boat_info,
            state: NavigatorState::new(),
            logs_cli: logs_cli,
        }
    }

    fn run_navigator_master_clock(&self) {
        let nav_params_check_pulse_clone = self.state.nav_params_check_pulse.clone();
        let route_check_pulse_clone = self.state.route_check_pulse.clone();
        let obstacles_check_pulse_clone = self.state.obstacles_check_pulse.clone();

        tokio::spawn(async move {
            loop {
                nav_params_check_pulse_clone.notify_waiters();
                sleep(Duration::from_millis(NAV_PARAMS_CHECK_DELAY)).await;
            }
        });

        tokio::spawn(async move {
            loop {
                route_check_pulse_clone.notify_waiters();
                sleep(Duration::from_millis(ROUTE_CHECK_DELAY)).await;
            }
        });

        tokio::spawn(async move {
            loop {
                obstacles_check_pulse_clone.notify_waiters();
                sleep(Duration::from_secs(1)).await;
            }
        });
    }

    pub fn set_voyage(&self, voyage: Voyage) {
        *self.voyage.lock().unwrap() = Some(voyage);
    }

    pub fn end_voyage(&self) {
        self.stop();
        *self.voyage.lock().unwrap() = None;
    }

    fn go_forward(&self) {
        self.serial_driver
            .lock()
            .unwrap()
            .change_motors_config(Some(100), Some(100));
    }

    fn go_backwards(&self) {
        self.serial_driver
            .lock()
            .unwrap()
            .change_motors_config(Some(-100), Some(-100));
    }

    async fn turn(&self, angle: i16) {
        let current = *self.boat_info.get_navigation_data().unwrap().true_heading() as i32;
        let target_heading = (current + angle as i32).rem_euclid(360) as u16;
        self.turn_to_heading(target_heading).await;
    }

    async fn turn_to_heading(&self, heading: u16) {
        self.logs_cli.send(LogEvent::System(
            format!("Rotation vers le cap {}°...", heading).yellow(),
        ));

        let mut prec_factor = 0;
        let mut factor = 0;

        loop {
            if self.is_on_heading(heading) {
                self.stop();
                self.logs_cli.send(LogEvent::System(
                    format!("Cap {}° atteint.", heading).green(),
                ));
                return;
            }

            let current_heading = match self.boat_info.get_navigation_data() {
                Ok(data) => *data.true_heading(),
                Err(_) => {
                    sleep(Duration::from_millis(NAV_PARAMS_CHECK_DELAY)).await;
                    continue;
                }
            };

            factor = (heading as i32 - current_heading as i32 + 540).rem_euclid(360) - 180;

            let (left_speed, right_speed) = if factor > 0 { (-100, 100) } else { (100, -100) };

            if prec_factor == 0 || prec_factor < 0 && factor > 0 || prec_factor > 0 && factor < 0 {
                self.serial_driver
                    .lock()
                    .unwrap()
                    .change_motors_config(Some(left_speed), Some(right_speed));

                prec_factor = factor;
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    fn stop(&self) {
        self.serial_driver
            .lock()
            .unwrap()
            .change_motors_config(Some(0), Some(0));
    }

    fn is_on_course(&self) -> bool {
        let nav_data = self.boat_info.get_navigation_data().unwrap();
        let (lon, lat) = (*nav_data.longitude() as u16, *nav_data.latitude() as u16);

        if let Some(seg) = self.current_segment_info() {
            return seg.distance_from_route((lon, lat))
                <= POSITION_UNCERTAINTY_RADIUS_MEDIUM as f64
                && self.is_on_heading(*seg.heading());
        }

        true
    }

    fn los_guidance(&self) -> ((u16, u16), u16) {
        let nav_data = self.boat_info.get_navigation_data().unwrap();
        let (current_lon, current_lat) =
            (*nav_data.longitude() as u16, *nav_data.latitude() as u16);

        if let Some(seg) = self.current_segment_info() {
            let (proj_lon, proj_lat) = seg.orthogonal_projection((current_lon, current_lat));

            let seg_heading_rad = (*seg.heading() as f64).to_radians();
            let lookahead_distance = 100.0;

            let trgt_lon = proj_lon as f64 + (lookahead_distance * seg_heading_rad.sin());
            let trgt_lat = proj_lat as f64 - (lookahead_distance * seg_heading_rad.cos());

            let dx = trgt_lon - current_lon as f64;
            let dy = trgt_lat - current_lat as f64;
            let trgt_heading_raw = dx.atan2(-dy).to_degrees();

            let trgt_heading = ((trgt_heading_raw + 360.0) % 360.0).round() as u16;

            return (
                (trgt_lon.round() as u16, trgt_lat.round() as u16),
                trgt_heading,
            );
        }

        ((current_lon, current_lat), 0)
    }

    fn has_reached_point(&self, p: (u16, u16)) -> bool {
        let nav_data = self.boat_info.get_navigation_data().unwrap();
        let (lon, lat) = (*nav_data.longitude() as i32, *nav_data.latitude() as i32);

        let distance = (((lon - p.0 as i32).pow(2) + (lat - p.1 as i32).pow(2)) as f64).sqrt();

        distance <= POSITION_UNCERTAINTY_RADIUS_MEDIUM as f64
    }

    async fn correct_course(&mut self) {
        self.state.is_correcting_course = true;

        let (trgt_pos, trgt_heading) = self.los_guidance();

        self.turn_to_heading(trgt_heading).await;
        self.go_forward();

        while !self.has_reached_point(trgt_pos) {
            sleep(Duration::from_millis(NAV_PARAMS_CHECK_DELAY)).await; // ATTENTION, C'EST DANGEREUX ! A TERME, IL FAUDRAIT JUSTE UPDATE LE VOYAGE AVEC LA MANOEUVRE DE CORRECTION, ET LAISSE LE NAVIGATEUR S'OCCUPER DE LA MANOEUVRE
        }

        self.stop();

        if let Some(seg) = self.current_segment_info() {
            self.turn_to_heading(*seg.heading()).await;
            self.go_forward();
        }

        self.state.is_correcting_course = false;
    }

    fn is_on_heading(&self, heading: u16) -> bool {
        let current_heading = *self.boat_info.get_navigation_data().unwrap().true_heading();
        let diff = current_heading.abs_diff(heading);
        let shortest_diff = if diff > 180 { 360 - diff } else { diff };

        shortest_diff <= ANGLE_UNCERTAINTY_RADIUS_MEDIUM
    }

    fn current_segment_info(&self) -> Option<VoyageSegment> {
        let mut voyage_guard = self.voyage.lock().unwrap();
        if let Some(voyage) = voyage_guard.as_mut() {
            Some(voyage.get_current_segment().clone())
        } else {
            None
        }
    }

    fn distance_from_current_segment_end(&mut self) -> Option<u16> {
        let mut voyage_guard = self.voyage.lock().unwrap();
        if let Some(voyage) = voyage_guard.as_mut() {
            if let Ok(nav) = self.boat_info.get_navigation_data() {
                Some(
                    voyage
                        .get_current_segment()
                        .distance_from_end((*nav.longitude() as u16, *nav.latitude() as u16)),
                )
            } else {
                None
            }
        } else {
            None
        }
    }

    async fn run_voyage(&mut self) {
        self.logs_cli
            .send(LogEvent::System("Exécution du voyage...".yellow()));

        loop {
            if let Some(seg) = self.current_segment_info() {
                self.turn_to_heading(*seg.heading()).await;
                self.go_forward();

                loop {
                    tokio::select! {
                        _ = self.state.nav_params_check_pulse.notified() => {
                            let nav_data = self.boat_info.get_navigation_data().unwrap();
                            let (lon, lat) = (*nav_data.longitude() as u16, *nav_data.latitude() as u16);
                            self.state.update((lon, lat));
                        },
                        _ = self.state.route_check_pulse.notified() => {
                            if !self.state.is_correcting_course {
                                let dist_to_seg_end = self.distance_from_current_segment_end();

                                match dist_to_seg_end {
                                    Some(d) if d <= POSITION_UNCERTAINTY_RADIUS_MEDIUM => {
                                        self.stop();

                                        let mut guard = self.voyage.lock().unwrap();

                                        if let Some(voyage) = guard.as_mut() {
                                            if voyage.next_segment().is_none() {
                                                *guard = None;

                                                self.logs_cli
                                                    .send(LogEvent::System("Exécution du voyage terminée.".yellow()));
                                            }
                                        }
                                        break;
                                    }
                                    Some(d) if d > POSITION_UNCERTAINTY_RADIUS_MEDIUM => {
                                        if !self.is_on_course() {
                                            self.correct_course().await;
                                        }
                                    }
                                    Some(_) | None => {
                                        self.stop();
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                self.logs_cli.send(LogEvent::System(
                    "Exécution du voyage terminée : aucun ordre de voyage en mémoire.".red(),
                ));

                return;
            }
        }
    }

    pub fn start(&self) {
        self.logs_cli
            .send(LogEvent::System("Lancement du navigateur...".yellow()));

        let mut self_clone = self.clone();
        let driver = self.serial_driver.clone();

        driver.lock().unwrap().start();
        self.run_navigator_master_clock();

        tokio::spawn(async move {
            loop {
                let has_voyage = self_clone.voyage.lock().unwrap().is_some();
                if has_voyage {
                    self_clone.run_voyage().await;
                } else {
                    sleep(Duration::from_millis(500)).await;
                }
            }
        });

        self.logs_cli
            .send(LogEvent::System("Navigateur lancé.".yellow()));
    }
}
