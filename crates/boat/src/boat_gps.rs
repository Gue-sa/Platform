use crate::{common::constants::BOAT_IPV4, systemstate::SystemState};
use colored::Colorize;
use shared::{bitpacker::BitPacker, boat_info::BoatInfo, common::types::LogEvent, config::Config};
use std::{net::Ipv4Addr, sync::Arc, time::Duration, u32};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
    time::interval,
};

pub struct BoatGps {
    boat_info: Arc<BoatInfo>,
    rx: Receiver<BitPacker>,
    antenna_tx: Sender<BitPacker>,
    system_state: Arc<SystemState>,
    logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
}

impl BoatGps {
    pub fn init(
        boat_info: Arc<BoatInfo>,
        rx: Receiver<BitPacker>,
        antenna_tx: Sender<BitPacker>,
        sys_state: Arc<SystemState>,
        logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
    ) -> Self {
        Self {
            boat_info: boat_info,
            rx: rx,
            antenna_tx: antenna_tx,
            system_state: sys_state,
            logs_cli_tx: logs_cli_tx,
        }
    }

    fn logs_cli_tx(&self) -> std::sync::mpsc::Sender<LogEvent> {
        self.logs_cli_tx.clone()
    }

    async fn run_gps(&mut self) {
        self.logs_cli_tx()
            .send(LogEvent::System("Lancement du GPS...".yellow()));

        let mut interval = interval(Duration::from_millis(
            //*Config::load().unwrap().gps_refresh_delay(),
            100,
        ));

        self.logs_cli_tx()
            .send(LogEvent::System("GPS lancé.".yellow()));

        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => {
                    if let Ok(heading) = msg.extract_int::<u32>(None, Some(31)) && let Ok(latitude) = msg.extract_int::<u32>(Some(32), Some(63)) && let Ok(longitude) = msg.extract_int::<u32>(Some(64), Some(95)) {
                        let _ = self.boat_info.update_positon(Some(latitude), Some(longitude), Some(heading as u16));

                        self.logs_cli_tx().send(LogEvent::Gps(format!("Position mise à jour depuis le GPS : ({latitude}, {longitude}, {heading})°").green()));
                    } else {
                        self.logs_cli_tx().send(LogEvent::Gps(format!("Positionnement GPS malformé reçu : {}", msg.to_bin_str()).red()));
                    }
                },
                _ = interval.tick() => {
                    let _ = self.antenna_tx.send(BitPacker::from_int(u32::from_be_bytes(BOAT_IPV4.octets()), Some(32))).await;

                    self.logs_cli_tx().send(LogEvent::Gps("Demande de positionnement envoyée à la capitainerie.".green()));
                }
            };
        }
    }

    pub fn start(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.run_gps().await;

            self.logs_cli_tx().send(LogEvent::System(
                "Le GPS s'est arrêté de façon inattendue. Veuillez redémarrer le GPS manuellement."
                    .red(),
            ));
        })
    }
}
