use std::sync::{Arc, mpsc::{Receiver, Sender, channel}};

use crate::{common::types::Packet, gps::Gps, station::Station};

pub struct  Harbourmaster {
    pub gps: Arc<Gps>,
    pub station: Arc<Station>,
    pub station_tx: Sender<Packet>,
    pub tx: Sender<Packet>,
    rx: Receiver<Packet>
}


impl Harbourmaster {
    pub fn init() -> Self {
        let (station_tx, station_rx) = channel::<Packet>();
        let (tx, rx) = channel::<Packet>();

        Self {
            gps: Arc::new(Gps::init()),
            station: Arc::new(Station::init(station_tx.clone(), station_rx, tx.clone())),
            station_tx: station_tx.clone(),
            tx: tx.clone(),
            rx: rx
        }
    }


    pub fn start(self: Arc<Self>) -> () {
        Arc::clone(&self.gps).start();
        Arc::clone(&self.station).start();
    }
}