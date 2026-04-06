use std::{sync::{Arc, Mutex, mpsc::{Receiver, Sender}}, thread, time::Duration};

use colored::*;

use crate::{boat_info::BoatInfo, common::{bitpacker::BitPacker, constants::BOAT_IP, utils::log}};


pub struct Gps {
    boat_info: Arc<BoatInfo>,
    rx: Mutex<Receiver<BitPacker>>,
    tx: Sender<BitPacker>,
    antenna_tx: Sender<BitPacker>
}


impl Gps {
    pub fn init(boat_info: Arc<BoatInfo>, rx: Receiver<BitPacker>, tx: Sender<BitPacker>, antenna_tx: Sender<BitPacker>) -> Self {
        Self {
            boat_info: boat_info,
            rx: Mutex::new(rx),
            tx: tx,
            antenna_tx: antenna_tx
        }
    }


    pub fn listen(self: Arc<Self>) -> () {
        tokio::spawn(async move {
            if let Ok(rx_guard) = self.rx.lock() {
                loop {
                    for msg in rx_guard.try_iter() {
                        let latitude: u32 = msg.extract_int::<u32>(None, Some(31)).unwrap();
                        let longitude: u32 = msg.extract_int::<u32>(Some(32), None).unwrap();
                        
                        self.boat_info.update_positon(Some(latitude), Some(longitude));

                        log(format!("Position mise à jour depuis le GPS : {latitude} | {longitude}").white().italic());
                    }  
                }
            }
        });
    }


    pub fn updater(self: Arc<Self>) -> () {
        tokio::spawn(async move {
            loop {
                let _ = self.antenna_tx.send(BitPacker::from_str(&BOAT_IP.to_string(), None).unwrap());
                thread::sleep(Duration::from_secs(1));
            }
        });
    }


    pub fn start(self: Arc<Self>) -> () {
        Arc::clone(&self).listen();
        self.updater();
    }
}