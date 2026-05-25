use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use shared::boat_info::BoatInfo;
use tokio::{sync::Notify, time::sleep};

use crate::{
    serial_driver::SerialDriver,
    voyage::{Voyage, VoyageSegment},
};

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

        let mut estimated_heading = 0;

        for window in self.positions_history.windows(2) {
            let (lat1, lon1) = window[0];
            let (lat2, lon2) = window[1];

            let window_heading = (lon2 as f64 - lon1 as f64)
                .atan2(lat2 as f64 - lat1 as f64)
                .to_degrees()
                .round() as u16;
            estimated_heading = (window_heading + estimated_heading) / 2;
        }

        estimated_heading
    }

    fn calculate_speed_estimate(&self) -> u8 {
        if self.positions_history.len() < 2 {
            return self.speed_estimate;
        }

        let mut estimated_speed = 0;

        for window in self.positions_history.windows(2) {
            let (lat1, lon1) = window[0];
            let (lat2, lon2) = window[1];

            let distance = (((lon2 as u16 - lon1 as u16) as f64).powf(2.)
                + ((lat2 as u16 - lat1 as u16) as f64).powf(2.))
            .sqrt();

            estimated_speed = ((distance as f64 / 0.001).round() as u8 + estimated_speed) / 2;
        }

        estimated_speed as u8
    }

    pub fn update(&mut self, new_pos: (u16, u16)) {
        self.positions_history.push(new_pos);

        if self.positions_history.len() > 5 {
            self.positions_history.remove(0);
        }

        let new_heading_estimate = self.calculate_heading_estimate();
        let new_speed_estimate = self.calculate_speed_estimate();

        self.heading_estimate = new_heading_estimate;
        self.speed_estimate = new_speed_estimate;
    }
}

impl Navigator {
    pub fn init(
        voyage_opt: Arc<Mutex<Option<Voyage>>>,
        serial_driver: Arc<Mutex<SerialDriver>>,
        boat_info: Arc<BoatInfo>,
    ) -> Self {
        Self {
            voyage: voyage_opt,
            serial_driver: serial_driver,
            boat_info: boat_info,
            state: NavigatorState::new(),
        }
    }

    fn run_navigator_master_clock(&self) {
        let nav_params_check_pulse_clone = self.state.nav_params_check_pulse.clone();
        let route_check_pulse_clone = self.state.route_check_pulse.clone();
        let obstacles_check_pulse_clone = self.state.obstacles_check_pulse.clone();

        tokio::spawn(async move {
            loop {
                nav_params_check_pulse_clone.notify_waiters();
                sleep(Duration::from_secs(3)).await;
            }
        });

        tokio::spawn(async move {
            loop {
                route_check_pulse_clone.notify_waiters();
                sleep(Duration::from_millis(200)).await;
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
        let target_heading =
            (self.boat_info.get_navigation_data().unwrap().true_heading() + angle as u16) % 360;
        self.turn_to_heading(target_heading).await;
    }

    async fn turn_to_heading(&self, heading: u16) {
        let current_heading = *self.boat_info.get_navigation_data().unwrap().true_heading();

        let diff = current_heading.abs_diff(heading);

        if diff < 5 {
            return;
        }

        if (current_heading + 360 - heading) % 360 < 180 {
            self.serial_driver
                .lock()
                .unwrap()
                .change_motors_config(Some(-100), Some(100));
        } else {
            self.serial_driver
                .lock()
                .unwrap()
                .change_motors_config(Some(100), Some(-100));
        }

        while !self.is_on_heading(heading) {
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
        let (lat, lon) = (*nav_data.latitude() as u16, *nav_data.longitude() as u16);

        if let Some(seg) = self.current_segment_info() {
            return seg.distance_from_route((lat, lon)) < 50.;
        }

        true
    }

    fn los_guidance(&self) -> ((u16, u16), u16) {
        let nav_data = self.boat_info.get_navigation_data().unwrap();
        let (current_lat, current_lon) =
            (*nav_data.latitude() as u16, *nav_data.longitude() as u16);

        if let Some(seg) = self.current_segment_info() {
            let (proj_lat, proj_lon) = seg.orthogonal_projection((current_lat, current_lon));

            let seg_heading_rad = (*seg.heading() as f64).to_radians();
            let lookahead_distance = 100.0;

            let trgt_lon = proj_lon as f64 + (lookahead_distance * seg_heading_rad.sin());
            let trgt_lat = proj_lat as f64 + (lookahead_distance * seg_heading_rad.cos());

            let trgt_heading = (trgt_lon - current_lon as f64)
                .atan2(trgt_lat - current_lat as f64)
                .to_degrees()
                .round() as u16;

            return (
                (trgt_lat.round() as u16, trgt_lon.round() as u16),
                trgt_heading,
            );
        }

        ((current_lat, current_lon), 0)
    }

    async fn correct_course(&mut self) {
        self.state.is_correcting_course = true;

        let (trgt_pos, trgt_heading) = self.los_guidance();

        self.turn_to_heading(trgt_heading).await;
        self.go_forward();

        while !self.has_reached_point(trgt_pos) {
            sleep(Duration::from_millis(200)).await;
        }

        self.stop();

        if let Some(seg) = self.current_segment_info() {
            self.turn_to_heading(*seg.heading()).await;

            if !(seg.distance_from_end(trgt_pos) < 50) {
                self.go_forward();
            }
        }

        self.state.is_correcting_course = false;
    }

    fn has_reached_point(&self, p: (u16, u16)) -> bool {
        let nav_data = self.boat_info.get_navigation_data().unwrap();
        let (lat, lon) = (*nav_data.latitude(), *nav_data.longitude());

        let distance = ((((lon as u16 - p.0) as i32).pow(2) + ((lat as u16 - p.1) as i32).pow(2))
            as f64)
            .sqrt();

        distance < 50.0
    }

    fn is_on_heading(&self, heading: u16) -> bool {
        let current_heading = *self.boat_info.get_navigation_data().unwrap().true_heading();

        let diff = current_heading.abs_diff(heading);

        diff < 5
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
        loop {
            if let Some(seg) = self.current_segment_info() {
                self.turn_to_heading(*seg.heading()).await;
                self.go_forward();

                loop {
                    tokio::select! {
                        _ = self.state.nav_params_check_pulse.notified() => {
                            let nav_data = self.boat_info.get_navigation_data().unwrap();
                            let (lat, lon) = (*nav_data.latitude() as u16, *nav_data.longitude() as u16);
                            self.state.update((lat, lon));
                        },
                        _ = self.state.route_check_pulse.notified() => {
                            if !self.state.is_correcting_course {
                                let dist_to_seg_end = self.distance_from_current_segment_end();

                                match dist_to_seg_end {
                                    Some(d) if d < 50 => {
                                        let mut guard = self.voyage.lock().unwrap();
                                        if let Some(voyage) = guard.as_mut() {
                                            if voyage.next_segment().is_none() {
                                                *guard = None;
                                                return;
                                            }
                                        }
                                        break;
                                    }
                                    Some(_) => {}
                                    None => return
                                }

                                if !self.is_on_course() {
                                    self.correct_course().await;
                                }
                            }
                        }
                        /*,
                        _ = self.state.obstacles_check_pulse.notified() => {
                            todo!()
                        }
                        */
                    }
                }
            } else {
                return;
            }
        }
    }

    pub fn start(&self) {
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
    }
}
