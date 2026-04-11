use crate::{common::types::Packet, gps::Gps, station::Station};

use tokio::sync::{mpsc::{Receiver, Sender, channel}, Semaphore};


pub struct  Harbourmaster {
    pub gps: Gps,
    pub station: Station,
    pub station_tx: Sender<Packet>,
    rx: Receiver<Packet>
}


impl Harbourmaster {
    pub async fn init() -> Self {
        let (station_tx, station_rx) = channel::<Packet>(Semaphore::MAX_PERMITS);
        let (tx, rx) = channel::<Packet>(Semaphore::MAX_PERMITS);

        Self {
            gps: Gps::init().await,
            station: Station::init(station_tx.clone(), station_rx, tx.clone()).await,
            station_tx: station_tx,
            rx: rx
        }
    }


    pub async fn start(self) -> () {
        let _ = &self.gps.start().await;
        let _ = &self.station.start().await;
    }
}