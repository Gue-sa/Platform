use std::{sync::{Arc, Mutex, mpsc::{Receiver, Sender}}, thread, time::Duration};

use colored::*;

use crate::{boat_info::BoatInfo, common::{constants::BOAT_IP, utils::log}};


pub struct Gps {
    pub boat_info: Arc<BoatInfo>,
    rx: Mutex<Receiver<String>>,
    pub tx: Sender<String>,
    pub antenna_tx: Sender<String>
}


impl Gps {
    pub fn init(boat_info: Arc<BoatInfo>, rx: Receiver<String>, tx: Sender<String>, antenna_tx: Sender<String>) -> Self {
        Self {
            boat_info: boat_info,
            rx: Mutex::new(rx),
            tx: tx,
            antenna_tx: antenna_tx
        }
    }


    pub fn listen(self: Arc<Self>) -> () {
        thread::spawn(move || {
            if let Ok(rx_guard) = self.rx.lock() {
                loop {
                    for msg in rx_guard.try_iter() {
                        let mut parts: std::str::Split<'_, &str> = msg.split(" | ");
                        let latitude: u32 = parts.next().unwrap().parse().unwrap();
                        let longitude: u32 = parts.next().unwrap().parse().unwrap();
                        
                        self.boat_info.set_latitude(Some(latitude));
                        self.boat_info.set_longitude(Some(longitude));

                        log(format!("Position mise à jour depuis le GPS : {}", msg).white().italic());
                    }  
                }
            }
        });
    }


    pub fn updater(self: Arc<Self>) -> () {
        thread::spawn(move || {
            loop {
                let _ = self.antenna_tx.send(BOAT_IP.to_string());
                thread::sleep(Duration::from_secs(1));
            }
        });
    }


    pub fn start(self: Arc<Self>) -> () {
        Arc::clone(&self).listen();
        self.updater();
    }
}