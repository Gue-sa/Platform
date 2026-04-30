use crate::{
    common::{
        constants::BOAT_IPV4,
        utils::{gps_log, system_log},
    },
    systemstate::SystemState,
};
use colored::Colorize;
use shared::{bitpacker::BitPacker, boat_info::BoatInfo};
use std::{net::Ipv4Addr, sync::Arc, u32};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
    time::{Duration, Interval, interval},
};

pub struct BoatGps {
    boat_info: Arc<BoatInfo>,
    rx: Receiver<BitPacker>,
    antenna_tx: Sender<BitPacker>,
    system_state: Arc<SystemState>,
}

impl BoatGps {
    pub fn init(
        boat_info: Arc<BoatInfo>,
        rx: Receiver<BitPacker>,
        antenna_tx: Sender<BitPacker>,
        sys_state: Arc<SystemState>,
    ) -> Self {
        Self {
            boat_info: boat_info,
            rx: rx,
            antenna_tx: antenna_tx,
            system_state: sys_state,
        }
    }

    async fn run_gps(&mut self) -> () {
        system_log("Lancement du GPS...".yellow());

        let mut interval: Interval = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => {
                    if let Ok(latitude) = msg.extract_int::<u32>(None, Some(31)) && let Ok(longitude) = msg.extract_int::<u32>(Some(32), None) {
                        let _ = self.boat_info.update_positon(Some(latitude), Some(longitude));

                        gps_log(format!("Position mise à jour depuis le GPS : {latitude} | {longitude}").white().italic());
                    }
                },
                _ = interval.tick() => {
                    let _ = self.antenna_tx.send(BitPacker::from_int(Ipv4Addr::to_bits(BOAT_IPV4), Some(32))).await;

                    gps_log("Demande de positionnement envoyée à la capitainerie.".white());
                }
            };
        }
    }

    pub fn start(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.run_gps().await;

            system_log("Le GPS s'est arrêté de façon inattendue. Veuillez redémarrer le GPS manuellement.".red());
        })
    }
}
