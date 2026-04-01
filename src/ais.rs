use std::{sync::{Arc, Mutex, RwLock, atomic::{AtomicI64, AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering::Relaxed}, mpsc::{Receiver, Sender}}, thread::{self, sleep}, time};

use colored::*;
use rand::{Rng, seq::IndexedRandom};

use crate::{antenna::Packet, boat_info::BoatInfo, boats_registry::{self, BoatsInfoRegistry}, common::{constants::*, types::*, utils::*}, impl_atomic_access, impl_rwlock_access, impl_mutex_access, impl_arc_access, message::{CommunicationState, Message}, slots_map::SlotsMap};


pub struct AisState {
    boat_info: Arc<BoatInfo>,
    boats_registry: Arc<RwLock<BoatsInfoRegistry>>,
    slots_map: SlotsMap,

    recv_stations: AtomicU8,
    sync_state: AtomicU8,
    last_msg5_timestamp: AtomicI64,

    sotdma_nss: Mutex<Option<u16>>,
    sotdma_ns: Mutex<Option<u16>>,
    sotdma_nts: Mutex<Option<u16>>,
    sotdma_ri: AtomicU32,
    sotdma_rr: Mutex<f64>,
    sotdma_ni: AtomicU16,
    sotdma_si: AtomicU16,
    sotdma_tmo_min: AtomicU8,
    sotdma_tmo_max: AtomicU8,
    sotdma_t_counter: AtomicU64
}


pub struct AisRunner {
    state: AisState,
    ais_tx: Sender<Packet>,
    ais_rx: Mutex<Receiver<Packet>>,
    c_87_b_tx: Sender<String>,
    c_88_b_tx: Sender<String>
}


impl AisState {
    pub fn init(boat_info: Arc<BoatInfo>, boats_registry: Arc<RwLock<BoatsInfoRegistry>>) -> Self {
        let mmsi: u32 = boat_info.get_static_data().mmsi;

        Self {
            boat_info: boat_info,
            boats_registry: boats_registry,
            slots_map: SlotsMap::init(mmsi),
            recv_stations: AtomicU8::new(0),
            sync_state: AtomicU8::new(0),
            last_msg5_timestamp: AtomicI64::new(-1),
            sotdma_nss: Mutex::from(None),
            sotdma_ns: Mutex::from(None),
            sotdma_nts: Mutex::from(None),
            sotdma_ri: AtomicU32::from(10),
            sotdma_rr: Mutex::from(6.),
            sotdma_ni: AtomicU16::from(375),
            sotdma_si: AtomicU16::from(75),
            sotdma_tmo_min: AtomicU8::from(3),
            sotdma_tmo_max: AtomicU8::from(7),
            sotdma_t_counter: AtomicU64::from(1)
        }
    }


    pub fn slots_map(&self) -> &SlotsMap {
        &self.slots_map
    }


    pub fn increase_t_counter(&self) -> () {
        self.sotdma_t_counter.fetch_add(1, Relaxed);
    }


    pub fn decrease_t_counter(&self) -> () {
        self.sotdma_t_counter.fetch_sub(1, Relaxed);
    }
    

    impl_arc_access!(boat_info, Arc<BoatInfo>, boat_info, set_boat_info);
    impl_rwlock_access!(boats_registry, BoatsInfoRegistry, boats_registry, set_boats_registry);

    impl_atomic_access!(recv_stations, u8, recv_stations, set_recv_stations);
    impl_atomic_access!(sync_state, u8, sync_state, set_sync_state);
    impl_atomic_access!(last_msg5_timestamp, i64, last_msg5_timestamp, set_last_msg5_timestamp);
    impl_atomic_access!(sotdma_ri, u32, ri, set_ri);
    impl_atomic_access!(sotdma_ni, u16, ni, set_ni);
    impl_atomic_access!(sotdma_si, u16, si, set_si);
    impl_atomic_access!(sotdma_tmo_min, u8, tmo_min, set_tmo_min);
    impl_atomic_access!(sotdma_tmo_max, u8, tmo_max, set_tmo_max);
    impl_atomic_access!(sotdma_t_counter, u64, t_counter, set_t_counter);

    impl_mutex_access!(sotdma_nss, Option<u16>, nss, set_nss);
    impl_mutex_access!(sotdma_ns, Option<u16>, ns, set_ns);
    impl_mutex_access!(sotdma_nts, Option<u16>, nts, set_nts);
    impl_mutex_access!(sotdma_rr, f64, rr, set_rr);
}


impl AisRunner {
    pub fn init(tx: Sender<Packet>, rx: Receiver<Packet>, c_87_b_tx: Sender<String>, c_88_b_tx: Sender<String>, boat_info: Arc<BoatInfo>, boats_registry: Arc<RwLock<BoatsInfoRegistry>>) -> Self {
        Self {
            state: AisState::init(boat_info, boats_registry),
            ais_tx: tx,
            ais_rx: Mutex::new(rx),
            c_87_b_tx: c_87_b_tx,
            c_88_b_tx: c_88_b_tx
        }
    }


    pub fn listen(self: Arc<Self>) -> () {
        thread::spawn(move || {
            loop {
                if let Ok(rx_guard) = self.ais_rx.lock() {
                    for packet in rx_guard.try_iter() {
                        match packet.channel {
                            Channel::C87B => self.handle_transmission(&packet.message, Channel::C87B),
                            Channel::C88B => self.handle_transmission(&packet.message, Channel::C88B),
                            _ => todo!()
                        }
                    }  
                }
            }
        });
    }


    pub fn handle_transmission(&self, msg: &str, channel: Channel) -> () {
        let t_s: u16 = SlotsMap::current_slot_number(channel);
        let (msg_type, _, communication_state, _, boat_info) = Message::parse(msg).unwrap();
        let boat_mmsi: u32 = boat_info.get_static_data().mmsi;
        let self_mmsi: u32 = self.state.boat_info().get_static_data().mmsi;
        
        if boat_mmsi != self_mmsi && IMPLEMENTED_MSGS.binary_search(&msg_type).is_ok() {
            if self.state.boats_registry().is_registered(&boat_mmsi) {
                self.state.boats_registry.write().unwrap().update(boat_info.clone());
            } else {
                self.state.boats_registry.write().unwrap().register(boat_info.clone());
            }

            let slots_map: &SlotsMap = self.state.slots_map();
            let t_s_owner: Option<u32> = slots_map.slot_owner(t_s);
            let t_s_timeout: Option<u8> = slots_map.slot_timeout(t_s);

            if t_s_owner.is_none() || t_s_owner == Some(self_mmsi) {
                if t_s_timeout.is_some() {
                    slots_map.use_slot(t_s);
                } else {
                    slots_map.mark_slot_as_used(t_s);
                }

                if [1, 2].binary_search(&msg_type).is_ok() {
                    let cs_timeout: u8 = communication_state.clone().unwrap().slot_timeout().unwrap();
                    
                    if t_s_owner.is_none() && cs_timeout > 0 {
                        slots_map.book_slot(t_s, self_mmsi, Some(cs_timeout), None);
                    } else if t_s_timeout.is_none() || cs_timeout > 0 {
                        slots_map.slots.write().unwrap()[t_s as usize].timeout = Some(cs_timeout);
                    } else if t_s_timeout == Some(0) || cs_timeout == 0 {
                        slots_map.release_slot(t_s);
                    }

                    if cs_timeout == 0 {
                        let cs_offset: u16 = communication_state.clone().unwrap().slot_offset().unwrap();
                        let rsv_s: u16 = SlotsMap::offseted_slot(t_s, cs_offset);

                        slots_map.book_slot(rsv_s, boat_mmsi, Some(cs_timeout), None);
                        slots_map.release_slot(t_s);
                    }
                } else if msg_type == 3 {
                    let cs_keep_flag: bool = communication_state.clone().unwrap().keep_flag.unwrap();
                    let cs_slot_increment: u16 = communication_state.clone().unwrap().slot_increment().unwrap();

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

            log(format!("Message {} reçu du navire {} : {:?}.", msg_type, boat_mmsi, boat_info.clone()).blue());
        }
    }


    pub fn wait_for_slot(slot_idx: u16) -> () { // A utiliser dans thread sender !
        let channel: Channel = if slot_idx < SLOTS_PER_MINUTE {Channel::C87B} else {Channel::C88B};
        while SlotsMap::current_slot_number(channel) != slot_idx {
            sleep(time::Duration::from_millis(1));
        }
    }


    pub fn wait_for_nts(&self) -> () {
        let nts: u16 = self.state.nts().unwrap();
        AisRunner::wait_for_slot(nts);
    }


    pub fn set_initial_nss_and_ns(&self) -> () {
        let initial_nss_and_ns: u16 = self.ratdma_slot_selection(Channel::C87B, 1).unwrap();
        self.state.set_nss(Some(initial_nss_and_ns));
        self.state.set_ns(Some(initial_nss_and_ns));
    }


    pub fn get_next_ns(&self) -> u16 {
        let nss: u16 = self.state.nss().unwrap();
        let t_counter: u64 = self.state.t_counter();
        let ni: u16 = self.state.ni();
        ((nss as u64 + t_counter * ni as u64) % SLOTS_PER_MINUTE as u64) as u16
    }


    pub fn set_next_ns(&self) -> () {
        let next_ns: u16 = self.get_next_ns();
        self.state.set_ns(Some(next_ns));
    }


    pub fn set_next_nts(&self) -> u16 {
        let nts: u16 = self.state.nts().unwrap_or(5500);

        let rsv_chn: Channel = if nts < SLOTS_PER_MINUTE { Channel::C88B } else { Channel::C87B };
        let ns: u16 = self.state.ns().unwrap();
        let si: u16 = self.state.si();
        let start_si: u16 = (ns + SLOTS_PER_MINUTE + SLOTS_PER_MINUTE - si.div_euclid(2)) % SLOTS_PER_MINUTE;
        
        let available_nts: Box<[u16]> = self.state.slots_map().scan_for_free_slots(Some(si), Some(start_si), None, rsv_chn).unwrap();
        let next_nts: u16 = *available_nts.choose(&mut rand::rng()).unwrap();

        let tmo_min: u8 = self.state.tmo_min();
        let tmo_max: u8 = self.state.tmo_max();

        let timeout: u8 = rand::rng().random_range(tmo_min..=tmo_max) as u8;

        let mmsi: u32 = self.state.boat_info().get_static_data().mmsi;
        self.state.slots_map().book_slot(next_nts, mmsi, Some(timeout), Some(false));
        
        next_nts
    }


    pub fn get_next_nts(&self) -> Result<u16, String> {
        let next_ns: u16 = self.get_next_ns();
        let si: u16 = self.state.si();
        let start_si: u16 = (next_ns + SLOTS_PER_MINUTE - si.div_euclid(2)) % SLOTS_PER_MINUTE;

        let available_ss: Box<[u16]> = self.state.slots_map().scan_for_self_owned_slots(Some(si), Some(start_si), Channel::Any)?;
        
        Ok(*available_ss.choose(&mut rand::rng()).unwrap())
    }


    pub fn send(&self, msg_type: u8, keep_flag: Option<bool>, offset: Option<u16>, slots_nbr: Option<u8>) -> () {
        let ant_tx:&Sender<String>  = if self.state.nts().unwrap() < SLOTS_PER_MINUTE {&self.c_87_b_tx} else {&self.c_88_b_tx};

        let sync_state: u8 = self.state.sync_state();
        let nts: u16 = self.state.nts().unwrap();

        let timeout: Option<u8> = self.state.slots_map().slot_timeout(nts);
        let recv_stations: u8 = self.state.recv_stations();

        let com_state: Option<CommunicationState> = if NO_CS_MSGS.binary_search(&msg_type).is_err() {Some(CommunicationState::init(msg_type, Some(sync_state), timeout, offset, Some(nts), Some(recv_stations), offset, slots_nbr, keep_flag))} else {None};

        let msg: Message = Message::init(None, Some(self.state.boat_info().as_ref().clone()), Some(msg_type), com_state).unwrap();
        
        let _ = ant_tx.send(msg.build());

        log(format!("Message {} envoyé avec succès sur le slot {}.", msg_type, self.state.nts().unwrap()).green());
    }


    pub fn ratdma_slot_selection(&self, chn: Channel, lme_rtpri: u8) -> Result<u16, &'static str> {
        let start_s: u16 = SlotsMap::current_slot_number(chn);
        
        let lme_rtes: u16 = SlotsMap::offseted_slot(start_s, 150);

        let slots_range: Box<[u16]> = self.state.slots_map().slots_idx_range(start_s, lme_rtes, chn);
        let mut candidates: Vec<u16> = Vec::from(self.state.slots_map().extract_available_slots_idx(slots_range));
    
        match candidates.len() {
            0 => Err("Aucun slot disponible."),
            _ => {
                let mut candidate: u16 = *candidates.choose(&mut rand::rng()).unwrap();

                let mut lme_rtcsc: u8 = candidates.len() as u8;
                let mut lme_rta: u8 = 0;

                let lme_rtps: f64 = 100. / lme_rtcsc as f64;
                let lme_rtp1: f64 = rand::rng().random_range(0.0..=100.0);
                let mut lme_rtp2: f64 = lme_rtps;
                let mut lme_rtpi: f64 = (100. - lme_rtp2) / lme_rtcsc as f64;

                while lme_rtp1 > lme_rtp2 as f64 {
                    lme_rtp2 += lme_rtpi;
                    lme_rtcsc -= 1;
                    lme_rta += 1;
                    lme_rtpi = (100. - lme_rtp2) / lme_rtcsc as f64;
                    candidates.retain(|c| *c != candidate);
                    candidate = *candidates.choose(&mut rand::rng()).unwrap();
                }

                Ok(candidate)
            }
        }
    }


    pub fn itdma(&self, t_s: u16, msg_type: u8, lme_itinc: u16, lme_itsl: u8, lme_itkp: bool) -> () {
        if ITDMA_CS_MSGS.binary_search(&msg_type).is_ok() {
            AisRunner::wait_for_slot(t_s);
            self.send(msg_type, Some(lme_itkp), Some(lme_itinc), Some(lme_itsl));
            self.state.slots_map().use_slot(t_s);
        } else if NO_CS_MSGS.binary_search(&msg_type).is_ok() {
            AisRunner::wait_for_slot(t_s);
            self.send(msg_type, None, None, None);
            self.state.slots_map().use_slot(t_s);
        }
    }


    pub fn sotdma_init(&self) -> () {
        sleep(time::Duration::from_millis(0));
    }


    pub fn sotdma_net_entry(&self) -> () {
        self.set_initial_nss_and_ns();
        let next_nts: u16 = self.set_next_nts();
        self.state.set_nts(Some(next_nts));
        log(format!("Premier NTS réservé : {}", self.state.nts().unwrap()).yellow());
        self.wait_for_nts();
    }


    pub fn sotdma_first_frame(&self) -> () {
        let mut virtual_offset: Option<u16> = None;
        let ref_nts: u16 = self.state.nts().unwrap();
        while virtual_offset.is_none() || virtual_offset != Some(0) {

            self.set_next_ns();
            let next_nts: u16 = self.set_next_nts();
            let si: u16 = self.state.si();
            let nts: u16 = self.state.nts().unwrap();
            let offset: u16 = SlotsMap::slot_offset(Some(nts), next_nts);
            virtual_offset = if SlotsMap::absolute_slot_distance(Some(next_nts), ref_nts) >= si {Some(offset)} else {Some(0)};
            let t_s: u16 = self.state.nts().unwrap();
            self.itdma(t_s, 3, virtual_offset.unwrap(), 1, true);
            self.state.increase_t_counter();

            log(format!("NTS réservé pour le prochain message 3 : {}.", next_nts).yellow());

            if virtual_offset.unwrap() != 0 {
                self.state.set_nts(Some(next_nts));
            } else {
                self.state.slots_map().release_slot(next_nts);
                self.state.set_nts(Some(ref_nts));
                self.state.decrease_t_counter();
            }
        }
    }


    pub fn sotdma_continuous(&self, msg_type: u8) -> () { // A refactor !
        if NO_CS_MSGS.binary_search(&msg_type).is_ok() {
            self.wait_for_nts();
            self.send(msg_type, None, None, None);
            let nts: u16 = self.state.nts().unwrap();
            self.state.slots_map().use_slot(nts);
            self.state.increase_t_counter();
            self.set_next_ns();

            match self.get_next_nts() {
                Ok(next_nts) => self.state.set_nts(Some(next_nts)),
                Err(e) => {
                    let next_nts: u16 = self.set_next_nts();
                    let nts: u16 = self.state.nts().unwrap();
                    let offset: u16 = SlotsMap::slot_offset(Some(next_nts), nts);

                    log(format!("NTS manquant détecté. Réservation du NTS {} pour le remplacer.", next_nts).yellow());

                    self.wait_for_nts();
                    self.itdma(nts, 3, offset, 1, true);
                    self.state.set_nts(Some(next_nts));
                }
            }
        } else if SOTDMA_CS_MSGS.binary_search(&msg_type).is_ok() {
            self.wait_for_nts();
            let nts: u16 = self.state.nts().unwrap();
            if self.state.slots_map().slot_timeout(nts) == Some(0) {
                let nts_channel: Channel = if nts < SLOTS_PER_MINUTE {Channel::C87B} else {Channel::C88B};
                let ns: u16 = self.state.ns().unwrap();
                let si: u16 = self.state.si();
                let start_si: u16 = (ns + SLOTS_PER_MINUTE - si.div_euclid(2)) % SLOTS_PER_MINUTE;
                let available_nts: Box<[u16]> = self.state.slots_map().scan_for_free_slots(Some(si), Some(start_si), None, nts_channel).unwrap();

                let new_nts: u16 = *available_nts.choose(&mut rand::rng()).unwrap();

                log(format!("NTS {} arrivé à expiration : remplacement par le slot {} après le prochain message.", nts, new_nts).yellow());

                let offset: u16 = SlotsMap::slot_offset(None, new_nts);
                self.send(msg_type, None, Some(offset), None);
                self.state.slots_map().use_slot(nts);
                self.state.increase_t_counter();
                self.set_next_ns();

                match self.get_next_nts() {
                    Ok(next_nts) => {
                        self.state.set_nts(Some(next_nts));
                        let tmo_min: u8 = self.state.tmo_min();
                        let tmo_max: u8 = self.state.tmo_max();
                        let timeout: u8 = rand::rng().random_range(tmo_min..=tmo_max);
                        let mmsi: u32 = self.state.boat_info().get_static_data().mmsi;
                        self.state.slots_map().book_slot(new_nts, mmsi, Some(timeout), None);
                    },
                    Err(e) => {
                        let next_nts: u16 = self.set_next_nts();

                        log(format!("NTS manquant détecté. Réservation du NTS {} pour le remplacer.", next_nts).yellow());

                        self.state.set_nts(Some(next_nts));
                        let tmo_min: u8 = self.state.tmo_min();
                        let tmo_max: u8 = self.state.tmo_max();
                        let timeout: u8 = rand::rng().random_range(tmo_min..=tmo_max);
                        let mmsi: u32 = self.state.boat_info().get_static_data().mmsi;
                        self.state.slots_map().book_slot(new_nts, mmsi, Some(timeout), None);
                    }
                }
            } else {
                self.send(msg_type, None, None, None);
                let nts: u16 = self.state.nts().unwrap();
                self.state.slots_map().use_slot(nts);
                self.state.increase_t_counter();
                self.set_next_ns();

                match self.get_next_nts() {
                    Ok(next_nts) => self.state.set_nts(Some(next_nts)),
                    Err(e) => {
                        let next_nts: u16 = self.set_next_nts();
                        let nts: u16 = self.state.nts().unwrap();
                        let offset: u16 = SlotsMap::slot_offset(Some(next_nts), nts);

                        log(format!("NTS manquant détecté. Réservation du NTS {} pour le remplacer.", next_nts).yellow());

                        self.wait_for_nts();
                        self.itdma(nts, 3, offset, 1, true);
                        self.state.set_nts(Some(next_nts));
                    }
                }
            }
        }
    }


    pub fn sotdma_change_rr(&self) -> () {
        todo!()
    }


    pub fn sotdma(self: Arc<Self>) -> () {
        thread::spawn(move || {
            log("Initialisation du SOTDMA...".yellow());
            self.sotdma_init();
            log("Initialisation du SOTMA terminée.".yellow());

            if self.state.ri() <= 120 {
                log("Entrée sur le réseau SOTDMA...".yellow());
                self.sotdma_net_entry();
                log("Entrée sur le réseau terminée.".yellow());
                log("Début de la première frame SOTMA...".yellow());
                self.sotdma_first_frame();
                log("Fin de la première frame.".yellow());
                log("Lancement de la phase continue SOTDMA.".yellow());

                loop {
                    let last_msg5_timestamp: i64 = self.state.last_msg5_timestamp();
                    if last_msg5_timestamp == -1 || get_timestamp(None) - last_msg5_timestamp >= 356 {
                        self.state.set_last_msg5_timestamp(get_timestamp(None));
                        self.sotdma_continuous(5);
                    } else {
                        self.sotdma_continuous(1);
                    }
                    sleep(time::Duration::from_millis(1));
                }
            }
        });
    }


    pub fn start(self: Arc<Self>) -> () {
        let runner_listen = Arc::clone(&self);
        let runner_sotdma = Arc::clone(&self);

        runner_listen.listen();
        runner_sotdma.sotdma();
    }
}


// ATTENTION : quand il spawn le thread listener d'une antenne, doit se charger lui-même de la loop !
