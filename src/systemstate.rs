use std::sync::{Arc, RwLock};

use colored::Colorize;
use tokio::sync::Notify;

use crate::common::utils::log;

pub struct SystemState {
    ais_emission_on: RwLock<bool>,
    gps_on: RwLock<bool>,
    can_navigate: RwLock<bool>,
    ais_notifier: Arc<Notify>,
    gps_notifier: Arc<Notify>,
    navigation_notifier: Arc<Notify>,
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            ais_emission_on: RwLock::new(false),
            gps_on: RwLock::new(false),
            can_navigate: RwLock::new(false),
            ais_notifier: Arc::new(Notify::new()),
            gps_notifier: Arc::new(Notify::new()),
            navigation_notifier: Arc::new(Notify::new()),
        }
    }

    pub fn ais_emitting(&self) -> bool {
        *self.ais_emission_on.read().unwrap()
    }

    pub fn gps_on(&self) -> bool {
        *self.gps_on.read().unwrap()
    }

    pub fn can_navigate(&self) -> bool {
        *self.can_navigate.read().unwrap()
    }

    pub fn stop_ais_emission(&self) -> () {
        let mut guard: std::sync::RwLockWriteGuard<'_, bool> =
            self.ais_emission_on.write().unwrap();
        if *guard {
            *guard = false;
        }
        log("Emission AIS en cours.".yellow());
    }

    pub fn start_ais_emission(&self) -> () {
        let mut guard: std::sync::RwLockWriteGuard<'_, bool> =
            self.ais_emission_on.write().unwrap();
        if !*guard {
            *guard = true;
        }
        log("Emission AIS interrompue.".yellow());
    }

    pub fn stop_gps(&self) -> () {
        let mut guard: std::sync::RwLockWriteGuard<'_, bool> = self.gps_on.write().unwrap();
        if *guard {
            *guard = false;
        }
        log("GPS démarré.".yellow());
    }

    pub fn start_gps(&self) -> () {
        let mut guard: std::sync::RwLockWriteGuard<'_, bool> =
            self.ais_emission_on.write().unwrap();
        if !*guard {
            *guard = true;
        }
        log("GPS éteint.".yellow());
    }

    pub fn start_navigation(&self) -> () {
        let mut guard: std::sync::RwLockWriteGuard<'_, bool> = self.can_navigate.write().unwrap();
        if *guard {
            *guard = false;
        }
        log("Navigation en cours.".yellow());
    }

    pub fn stop_navigation(&self) -> () {
        let mut guard: std::sync::RwLockWriteGuard<'_, bool> = self.can_navigate.write().unwrap();
        if !*guard {
            *guard = true;
        }
        log("Navigation interrompue.".yellow());
    }
}
