use std::{net::Ipv4Addr, sync::Arc, u32};

use tokio::{sync::mpsc::*, time::Duration};

use colored::*;

use crate::{boat_info::BoatInfo, common::{constants::{BOAT_IP, BOAT_IPV4}, types::GpsError, utils::log}, shared::bitpacker::BitPacker, systemstate::SystemState};


pub struct Gps {
    boat_info: Arc<BoatInfo>,
    rx: Receiver<BitPacker>,
    tx: Sender<BitPacker>,
    antenna_tx: Sender<BitPacker>,
    system_state: Arc<SystemState>
}


impl Gps {
    pub fn init(boat_info: Arc<BoatInfo>, rx: Receiver<BitPacker>, tx: Sender<BitPacker>, antenna_tx: Sender<BitPacker>, system_state: Arc<SystemState>) -> Self {
        Self {
            boat_info: boat_info,
            rx: rx,
            tx: tx,
            antenna_tx: antenna_tx,
            system_state: system_state
        }
    }


    pub async fn start(mut self) -> () {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    Some(msg) = self.rx.recv() => {
                        if let Ok(latitude) = msg.extract_int::<u32>(None, Some(31)) && let Ok(longitude) = msg.extract_int::<u32>(Some(32), None) {
                            self.boat_info.update_positon(Some(latitude), Some(longitude));

                            log(format!("Position mise à jour depuis le GPS : {latitude} | {longitude}").white().italic());
                        }
                    },
                    _ = interval.tick() => {
                        let _ = self.antenna_tx.send(BitPacker::from_int(Ipv4Addr::to_bits(BOAT_IPV4), Some(32))).await;
                    }
                };
            }
        });
    }
}