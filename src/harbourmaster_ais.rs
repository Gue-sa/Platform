use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU16},
};

use shared::{
    ais_message::Message,
    bitpacker::BitPacker,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::IMPLEMENTED_MSGS,
        types::{AisError, AisPacket, AisResult, Channel},
    },
    impl_atomic_access,
    slots_map::SlotsMap,
};
use tokio::sync::{Mutex, mpsc::*};

use colored::*;

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
            slots_map: SlotsMap::init(0b111111111111111111111111111111),
            recv_stations: AtomicU16::new(0),
            sync_state: AtomicU8::new(0),
        }
    }

    pub fn slots_map(&self) -> &SlotsMap {
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

    pub fn handle_transmission(&self, msg: BitPacker, channel: Channel) -> AisResult<Message> {
        let t_s: u16 = SlotsMap::current_slot_number(channel);
        let msg: Message = Message::from_bits(msg)?;
        let boat_mmsi: u32 = msg.boat_info.get_static_data().mmsi;

        if boat_mmsi != 0b111111111111111111111111111111
            && IMPLEMENTED_MSGS.binary_search(&msg.message_type).is_ok()
        {
            if self.state.boats_registry.is_registered(&boat_mmsi) {
                self.state.boats_registry.update(msg.boat_info.clone());
            } else {
                self.state.boats_registry.register(msg.boat_info.clone());
            }

            let slots_map: &SlotsMap = self.state.slots_map();
            let t_s_owner: Option<u32> = slots_map.slot_owner(t_s);
            let t_s_timeout: Option<u8> = slots_map.slot_timeout(t_s);

            if t_s_owner.is_none() || t_s_owner == Some(boat_mmsi) {
                if t_s_timeout.is_some() {
                    slots_map.use_slot(t_s);
                } else {
                    slots_map.mark_slot_as_used(t_s);
                }

                if [1, 2].binary_search(&msg.message_type).is_ok() {
                    let cs_timeout: u8 = msg
                        .communication_state
                        .clone()
                        .unwrap()
                        .slot_timeout
                        .unwrap();

                    if t_s_owner.is_none() && cs_timeout > 0 {
                        slots_map.book_slot(t_s, boat_mmsi, Some(cs_timeout), None);
                    } else if t_s_timeout.is_none() || cs_timeout > 0 {
                        slots_map.slots.write().unwrap()[t_s as usize].timeout = Some(cs_timeout);
                    } else if t_s_timeout == Some(0) || cs_timeout == 0 {
                        slots_map.release_slot(t_s);
                    }

                    if cs_timeout == 0 {
                        let cs_offset: u16 = msg
                            .communication_state
                            .clone()
                            .unwrap()
                            .slot_offset
                            .unwrap();
                        let rsv_s: u16 = SlotsMap::offseted_slot(t_s, cs_offset);

                        slots_map.book_slot(rsv_s, boat_mmsi, Some(cs_timeout), None);
                        slots_map.release_slot(t_s);
                    }
                } else if msg.message_type == 3 {
                    let cs_keep_flag: bool =
                        msg.communication_state.clone().unwrap().keep_flag.unwrap();
                    let cs_slot_increment: u16 = msg
                        .communication_state
                        .clone()
                        .unwrap()
                        .slot_increment
                        .unwrap();

                    if cs_keep_flag == false {
                        slots_map.release_slot(t_s);
                    } else if slots_map.slot_owner(t_s).is_none() && cs_keep_flag {
                        slots_map.book_slot(t_s, boat_mmsi, None, None);
                    }

                    if cs_slot_increment > 0 {
                        let rsv_s = SlotsMap::offseted_slot(t_s, cs_slot_increment);
                        slots_map.book_slot(rsv_s, boat_mmsi, None, None);
                    }
                }
            }
        } else {
            return Err(AisError::SelfEmittedMessage);
        }

        Ok(msg)
    }

    pub async fn start(self) -> () {
        let runner_arc: Arc<HarbourmasterAisRunner> = Arc::new(self);
        let c87b_runner_arc: Arc<HarbourmasterAisRunner> = runner_arc.clone();
        let c88b_runner_arc: Arc<HarbourmasterAisRunner> = runner_arc.clone();

        tokio::spawn(async move {
            loop {
                if let Some(packet) = runner_arc.ais_rx.lock().await.recv().await {
                    match packet.channel {
                        Channel::C87B => {
                            match c87b_runner_arc.handle_transmission(packet.message, Channel::C87B)
                            {
                                Ok(msg) => {
                                    println!(
                                        "{}",
                                        format!(
                                            "Message {} reçu du navire {} : {:?}.",
                                            msg.message_type,
                                            msg.boat_info.get_static_data().mmsi,
                                            msg.boat_info.clone()
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
                            }
                        }
                        Channel::C88B => {
                            match c88b_runner_arc.handle_transmission(packet.message, Channel::C88B)
                            {
                                Ok(msg) => {
                                    println!(
                                        "{}",
                                        format!(
                                            "Message {} reçu du navire {} : {:?}.",
                                            msg.message_type,
                                            msg.boat_info.get_static_data().mmsi,
                                            msg.boat_info.clone()
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
                            }
                        }
                        _ => todo!(),
                    }
                }
            }
        });
    }
}
