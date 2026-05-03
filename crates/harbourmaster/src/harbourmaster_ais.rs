use colored::*;
use shared::{
    ais_message::AisMessage,
    bitpacker::BitPacker,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::{HARBOURMASTER_MMSI, IMPLEMENTED_MSGS},
        errors::{AisError, AisResult},
        types::{AisPacket, Channel, LogEvent},
    },
    impl_atomic_access,
    slots_map::SlotsMap,
};
use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU16},
};
use tokio::{
    sync::{Mutex, mpsc::*},
    task::JoinHandle,
};

pub struct HarbourmasterAisState {
    boats_registry: Arc<BoatsInfoRegistry>,
    slots_map: SlotsMap,

    recv_stations: AtomicU16,
    sync_state: AtomicU8,
}

pub struct HarbourmasterAisRunner {
    state: HarbourmasterAisState,
    ais_rx: Mutex<Receiver<AisPacket>>,
    logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
}

impl HarbourmasterAisState {
    pub fn init(boats_registry: Arc<BoatsInfoRegistry>) -> Self {
        Self {
            boats_registry: boats_registry,
            slots_map: SlotsMap::new(HARBOURMASTER_MMSI),
            recv_stations: AtomicU16::new(0),
            sync_state: AtomicU8::new(0),
        }
    }

    fn slots_map(&self) -> &SlotsMap {
        &self.slots_map
    }

    impl_atomic_access!(recv_stations, u16, recv_stations, set_recv_stations);
    impl_atomic_access!(sync_state, u8, sync_state, set_sync_state);
}

impl HarbourmasterAisRunner {
    pub fn init(
        rx: Receiver<AisPacket>,
        boats_registry: Arc<BoatsInfoRegistry>,
        cli_tx: std::sync::mpsc::Sender<LogEvent>,
    ) -> Self {
        Self {
            state: HarbourmasterAisState::init(boats_registry),
            ais_rx: Mutex::new(rx),
            logs_cli_tx: cli_tx,
        }
    }

    fn logs_cli_tx(&self) -> std::sync::mpsc::Sender<LogEvent> {
        self.logs_cli_tx.clone()
    }

    fn handle_transmission(&self, msg: BitPacker, channel: Channel) -> AisResult<AisMessage> {
        let t_si = SlotsMap::current_si(channel);
        let msg = AisMessage::from_bits(msg)?;
        let boat_mmsi = *msg.boat_info().get_static_data()?.mmsi();

        if boat_mmsi != HARBOURMASTER_MMSI
            && IMPLEMENTED_MSGS.binary_search(msg.message_type()).is_ok()
        {
            if self.state.boats_registry.is_registered(&boat_mmsi) {
                self.state.boats_registry.update_from_ais_msg(&msg)?;
            } else {
                self.state.boats_registry.register(msg.boat_info())?;
            }

            let slots_map = self.state.slots_map();
            let t_si_owner = slots_map.slot_owner(t_si)?;
            let t_si_timeout = slots_map.slot_timeout(t_si)?;

            if t_si_owner.is_none() || t_si_owner == Some(boat_mmsi) {
                if t_si_timeout.is_some() {
                    slots_map.use_slot(t_si)?;
                } else {
                    slots_map.flag_slot_as_used(t_si)?;
                }

                if [1, 2].binary_search(msg.message_type()).is_ok() {
                    let com_state_timeout = *msg.communication_state()?.slot_timeout()?;

                    if t_si_owner.is_none() && com_state_timeout > 0 {
                        slots_map.book_slot(t_si, boat_mmsi, Some(com_state_timeout), None)?;
                    } else if t_si_timeout.is_none() || com_state_timeout > 0 {
                        slots_map.set_slot_timeout(t_si, Some(com_state_timeout))?;
                    } else if t_si_timeout == Some(0) || com_state_timeout == 0 {
                        slots_map.release_slot(t_si)?;
                    }

                    if com_state_timeout == 0 {
                        let cs_offset = *msg.communication_state()?.slot_offset()?;
                        let rsv_s = SlotsMap::offseted_si(t_si, cs_offset);

                        slots_map.book_slot(rsv_s, boat_mmsi, Some(com_state_timeout), None)?;
                        slots_map.release_slot(t_si)?;
                    }
                } else if *msg.message_type() == 3 {
                    let com_state_keep_flag = *msg.communication_state()?.keep_flag()?;
                    let com_state_slot_increment: u16 =
                        *msg.communication_state()?.slot_increment()?;

                    if com_state_keep_flag == false {
                        slots_map.release_slot(t_si)?;
                    } else if slots_map.slot_owner(t_si)?.is_none() && com_state_keep_flag {
                        slots_map.book_slot(t_si, boat_mmsi, None, None)?;
                    }

                    if com_state_slot_increment > 0 {
                        let rsv_s = SlotsMap::offseted_si(t_si, com_state_slot_increment);
                        slots_map.book_slot(rsv_s, boat_mmsi, None, None)?;
                    }
                }
            }
        } else {
            return Err(AisError::SelfEmittedMessage);
        }

        Ok(msg)
    }

    async fn run_listeners(&self) -> () {
        self.logs_cli_tx()
            .send(LogEvent::System("Lancement de l'écoute AIS...".yellow()));

        loop {
            let pck_opt = {
                let mut rx = self.ais_rx.lock().await;
                rx.recv().await
            };

            if let Some(pck) = pck_opt {
                match pck.channel {
                    Channel::C87B => match self.handle_transmission(pck.message, Channel::C87B) {
                        Ok(msg) => {
                            self.logs_cli_tx().send(LogEvent::Ais(
                                format!(
                                    "Message {} reçu du navire {}.",
                                    msg.message_type(),
                                    *msg.boat_info().get_static_data().unwrap().mmsi()
                                )
                                .green(),
                            ));
                        }
                        Err(e) => match e {
                            AisError::SelfEmittedMessage => {}
                            _ => {
                                self.logs_cli_tx().send(LogEvent::Ais(
                                    format!("{}", "Message corrompu reçu et ignoré.").red(),
                                ));
                            }
                        },
                    },
                    Channel::C88B => match self.handle_transmission(pck.message, Channel::C88B) {
                        Ok(msg) => {
                            self.logs_cli_tx().send(LogEvent::Ais(
                                format!(
                                    "Message {} reçu du navire {}.",
                                    msg.message_type(),
                                    *msg.boat_info().get_static_data().unwrap().mmsi()
                                )
                                .green(),
                            ));
                        }
                        Err(e) => match e {
                            AisError::SelfEmittedMessage => {}
                            _ => {
                                self.logs_cli_tx().send(LogEvent::Ais(
                                    format!("{}", "Message corrompu reçu et ignoré.").red(),
                                ));
                            }
                        },
                    },
                    _ => todo!(),
                }
            }
        }
    }

    pub fn start(self) -> (JoinHandle<()>, JoinHandle<()>) {
        self.logs_cli_tx()
            .send(LogEvent::System("Lancement de l'AIS...".yellow()));

        let listeners_runner_arc = Arc::new(self);
        let slots_map_cleanup_runner_arc: Arc<HarbourmasterAisRunner> =
            listeners_runner_arc.clone();

        slots_map_cleanup_runner_arc
            .logs_cli_tx()
            .send(LogEvent::System(
                "Lancement du nettoyage automatique de la table des slots AIS...".yellow(),
            ));

        (
            tokio::spawn(async move {
                slots_map_cleanup_runner_arc
                    .state
                    .slots_map()
                    .run_cleanup_task()
                    .await;

                slots_map_cleanup_runner_arc
                    .logs_cli_tx()
                    .send(LogEvent::System("Le daemon de nettoyage de la table des slots s'est arrêté de façon innatendue. Veuillez redémarrer l'AIS manuellement.".yellow()));
            }),
            tokio::spawn(async move {
                listeners_runner_arc.run_listeners().await;

                listeners_runner_arc.logs_cli_tx()
                    .send(LogEvent::System("L'écoute AIS s'est arrêtée de façon innatendue. Veuillez redémarrer l'AIS manuellement.".yellow()));
            }),
        )
    }
}
