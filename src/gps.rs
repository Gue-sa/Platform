use std::sync::Arc;

use tokio::{sync::mpsc::*, time::Duration};

use colored::*;

use crate::{boat_info::BoatInfo, common::{bitpacker::BitPacker, constants::BOAT_IP, utils::log}};


pub struct Gps {
    boat_info: Arc<BoatInfo>,
    rx: Receiver<BitPacker>,
    tx: Sender<BitPacker>,
    antenna_tx: Sender<BitPacker>
}


impl Gps {
    pub fn init(boat_info: Arc<BoatInfo>, rx: Receiver<BitPacker>, tx: Sender<BitPacker>, antenna_tx: Sender<BitPacker>) -> Self {
        Self {
            boat_info: boat_info,
            rx: rx,
            tx: tx,
            antenna_tx: antenna_tx
        }
    }


    pub async fn start(mut self) -> () {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    Some(msg) = self.rx.recv() => {
                        let latitude: u32 = msg.extract_int::<u32>(None, Some(31)).unwrap();
                        let longitude: u32 = msg.extract_int::<u32>(Some(32), None).unwrap();

                        self.boat_info.update_positon(Some(latitude), Some(longitude));

                        log(format!("Position mise à jour depuis le GPS : {latitude} | {longitude}").white().italic());
                    },
                    _ = interval.tick() => {
                        let _ = self.antenna_tx.send(BitPacker::from_str(&BOAT_IP.to_string(), None).unwrap()).await;
                    }
                };
            }
        });
    }
}