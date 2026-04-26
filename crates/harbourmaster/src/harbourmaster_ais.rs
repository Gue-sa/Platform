use colored::*;
use shared::{
    ais_message::AisMessage,
    bitpacker::BitPacker,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::{HARBOURMASTER_MMSI, IMPLEMENTED_MSGS},
        types::{AisError, AisPacket, AisResult, Channel},
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
    pub fn init(rx: Receiver<AisPacket>, boats_registry: Arc<BoatsInfoRegistry>) -> Self {
        Self {
            state: HarbourmasterAisState::init(boats_registry),
            ais_rx: Mutex::new(rx),
        }
    }

    fn handle_transmission(&self, msg: BitPacker, channel: Channel) -> AisResult<AisMessage> {
        let t_si: u16 = SlotsMap::current_si(channel);
        let msg: AisMessage = AisMessage::from_bits(msg)?;
        let boat_mmsi: u32 = *msg.boat_info().get_static_data().mmsi();

        if boat_mmsi != HARBOURMASTER_MMSI
            && IMPLEMENTED_MSGS.binary_search(msg.message_type()).is_ok()
        {
            if self.state.boats_registry.is_registered(&boat_mmsi) {
                self.state.boats_registry.update(msg.boat_info());
            } else {
                self.state.boats_registry.register(msg.boat_info());
            }

            let slots_map: &SlotsMap = self.state.slots_map();
            let t_s_owner: Option<u32> = slots_map.slot_owner(t_si);
            let t_s_timeout: Option<u8> = slots_map.slot_timeout(t_si);

            if t_s_owner.is_none() || t_s_owner == Some(boat_mmsi) {
                if t_s_timeout.is_some() {
                    slots_map.use_slot(t_si);
                } else {
                    slots_map.flag_slot_as_used(t_si);
                }

                if [1, 2].binary_search(msg.message_type()).is_ok() {
                    let com_state_timeout: u8 =
                        msg.communication_state().unwrap().slot_timeout().unwrap();

                    if t_s_owner.is_none() && com_state_timeout > 0 {
                        slots_map.book_slot(t_si, boat_mmsi, Some(com_state_timeout), None);
                    } else if t_s_timeout.is_none() || com_state_timeout > 0 {
                        slots_map.set_slot_timeout(t_si, Some(com_state_timeout));
                    } else if t_s_timeout == Some(0) || com_state_timeout == 0 {
                        slots_map.release_slot(t_si);
                    }

                    if com_state_timeout == 0 {
                        let com_state_offset: u16 =
                            msg.communication_state().unwrap().slot_offset().unwrap();
                        let rsv_s: u16 = SlotsMap::offseted_si(t_si, com_state_offset);

                        slots_map.book_slot(rsv_s, boat_mmsi, Some(com_state_timeout), None);
                        slots_map.release_slot(t_si);
                    }
                } else if *msg.message_type() == 3 {
                    let com_state_keep_flag: bool =
                        msg.communication_state().unwrap().keep_flag().unwrap();
                    let com_state_slot_increment: u16 =
                        msg.communication_state().unwrap().slot_increment().unwrap();

                    if com_state_keep_flag == false {
                        slots_map.release_slot(t_si);
                    } else if slots_map.slot_owner(t_si).is_none() && com_state_keep_flag {
                        slots_map.book_slot(t_si, boat_mmsi, None, None);
                    }

                    if com_state_slot_increment > 0 {
                        let rsv_s = SlotsMap::offseted_si(t_si, com_state_slot_increment);
                        slots_map.book_slot(rsv_s, boat_mmsi, None, None);
                    }
                }
            }
        } else {
            return Err(AisError::SelfEmittedMessage);
        }

        Ok(msg)
    }

    async fn run_listeners(&self) -> () {
        loop {
            let pck_opt = {
                let mut rx = self.ais_rx.lock().await;
                rx.recv().await
            };

            if let Some(pck) = pck_opt {
                match pck.channel {
                    Channel::C87B => match self.handle_transmission(pck.message, Channel::C87B) {
                        Ok(msg) => {
                            println!(
                                "{}",
                                format!(
                                    "Message {} reçu du navire {} : {:#?}.",
                                    msg.message_type(),
                                    *msg.boat_info().get_static_data().mmsi(),
                                    msg.boat_info()
                                )
                                .blue()
                            );
                        }
                        Err(e) => match e {
                            AisError::SelfEmittedMessage => {}
                            _ => {
                                println!("{}", "Message corrompu reçu et ignoré.".red());
                            }
                        },
                    },
                    Channel::C88B => match self.handle_transmission(pck.message, Channel::C88B) {
                        Ok(msg) => {
                            println!(
                                "{}",
                                format!(
                                    "Message {} reçu du navire {} : {:#?}.",
                                    *msg.message_type(),
                                    *msg.boat_info().get_static_data().mmsi(),
                                    msg.boat_info()
                                )
                                .blue()
                            );
                        }
                        Err(e) => match e {
                            AisError::SelfEmittedMessage => {}
                            _ => {
                                println!("{}", "Message corrompu reçu et ignoré.".red());
                            }
                        },
                    },
                    _ => todo!(),
                }
            }
        }
    }

    pub fn start(self) -> (JoinHandle<()>, JoinHandle<()>) {
        let listeners_runner_arc: Arc<HarbourmasterAisRunner> = Arc::new(self);
        let slots_map_cleanup_runner_arc: Arc<HarbourmasterAisRunner> =
            listeners_runner_arc.clone();

        (
            tokio::spawn(async move {
                slots_map_cleanup_runner_arc
                    .state
                    .slots_map()
                    .run_cleanup_task()
                    .await;
            }),
            tokio::spawn(async move {
                listeners_runner_arc.run_listeners().await;
            }),
        )
    }
}
