use std::sync::{Arc, atomic::{AtomicI64, AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering::Relaxed}};

use tokio::{sync::{Notify, mpsc::*}, time::{Duration, Instant, interval_at}};

use colored::*;
use rand::{Rng, seq::IndexedRandom};

use crate::{antenna::Packet, boat_info::BoatInfo, boats_registry::BoatsInfoRegistry, common::{bitpacker::BitPacker, constants::*, types::*, utils::*}, impl_arc_access, impl_atomic_access, impl_mutex_access, impl_option_access, impl_tokio_mutex_access, impl_tokio_rwlock_access, message::{CommunicationState, Message}, slots_map::SlotsMap};


pub struct AisState {
    c_87_b_tx: Sender<BitPacker>,
    c_88_b_tx: Sender<BitPacker>,

    clock_pulse: Notify,

    boat_info: Arc<BoatInfo>,
    boats_registry: BoatsInfoRegistry,
    slots_map: SlotsMap,

    recv_stations: AtomicU16,
    sync_state: AtomicU8,
    last_msg5_timestamp: AtomicI64,

    sotdma_nss: AtomicU16,
    sotdma_ns: AtomicU16,
    sotdma_nts: AtomicU16,
    sotdma_ri: AtomicU32,
    sotdma_rr: f64,
    sotdma_ni: AtomicU16,
    sotdma_si: AtomicU16,
    sotdma_tmo_min: AtomicU8,
    sotdma_tmo_max: AtomicU8,
    sotdma_t_counter: AtomicU64
}


pub struct AisRunner {
    state: AisState,
    ais_tx: Sender<Packet>,
    ais_rx: Receiver<Packet>
}


impl AisState {
    pub fn init(c_87_b_tx: Sender<BitPacker>, c_88_b_tx: Sender<BitPacker>, boat_info: Arc<BoatInfo>, boats_registry: BoatsInfoRegistry) -> Self {
        let mmsi: u32 = boat_info.get_static_data().mmsi;

        Self {
            c_87_b_tx: c_87_b_tx,
            c_88_b_tx: c_88_b_tx,
            clock_pulse: Notify::new(),
            boat_info: boat_info,
            boats_registry: boats_registry,
            slots_map: SlotsMap::init(mmsi),
            recv_stations: AtomicU16::new(0),
            sync_state: AtomicU8::new(0),
            last_msg5_timestamp: AtomicI64::new(-1),
            sotdma_nss: AtomicU16::new(u16::MAX),
            sotdma_ns: AtomicU16::new(u16::MAX),
            sotdma_nts: AtomicU16::new(u16::MAX),
            sotdma_ri: AtomicU32::from(10),
            sotdma_rr: 6.,
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

    impl_atomic_access!(recv_stations, u16, recv_stations, set_recv_stations);
    impl_atomic_access!(sync_state, u8, sync_state, set_sync_state);
    impl_atomic_access!(last_msg5_timestamp, i64, last_msg5_timestamp, set_last_msg5_timestamp);
    impl_atomic_access!(sotdma_ri, u32, ri, set_ri);
    impl_atomic_access!(sotdma_ni, u16, ni, set_ni);
    impl_atomic_access!(sotdma_si, u16, si, set_si);
    impl_atomic_access!(sotdma_tmo_min, u8, tmo_min, set_tmo_min);
    impl_atomic_access!(sotdma_tmo_max, u8, tmo_max, set_tmo_max);
    impl_atomic_access!(sotdma_t_counter, u64, t_counter, set_t_counter);

    impl_atomic_access!(sotdma_nss, u16, nss, set_nss);
    impl_atomic_access!(sotdma_ns, u16, ns, set_ns);
    impl_atomic_access!(sotdma_nts, u16, nts, set_nts);
}


impl AisRunner {
    pub fn init(tx: Sender<Packet>, rx: Receiver<Packet>, c_87_b_tx: Sender<BitPacker>, c_88_b_tx: Sender<BitPacker>, boat_info: Arc<BoatInfo>, boats_registry: BoatsInfoRegistry) -> Self {
        Self {
            state: AisState::init(c_87_b_tx.clone(), c_88_b_tx.clone(), boat_info, boats_registry),
            ais_tx: tx,
            ais_rx: rx
        }
    }


    pub async fn master_clock(state: Arc<AisState>) {
        let slot_duration_ns: u64 = 80_000_000 / 3;
        
        let now_utc: Duration = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        
        let ns_since_current_minute: u64 = (now_utc.as_nanos() % 60_000_000_000) as u64;
        let next_slot_start_ns: u64 = ((ns_since_current_minute / slot_duration_ns) + 1) * slot_duration_ns;
        let first_tick_delay: u64 = next_slot_start_ns - ns_since_current_minute;

        let first_tick: Instant = Instant::now() + Duration::from_nanos(first_tick_delay);
        let mut interval: tokio::time::Interval = interval_at(first_tick, Duration::from_nanos(slot_duration_ns));

        log("Horloge SOTDMA synchronisée sur l'heure UTC.".yellow());

        loop {
            interval.tick().await;
            state.clock_pulse.notify_waiters(); 
        }
    }


    pub fn handle_transmission(state: Arc<AisState>, msg: BitPacker , channel: Channel) -> () {
        let t_s: u16 = SlotsMap::current_slot_number(channel);
        let msg: Message = Message::from_bits(msg).unwrap();
        let boat_mmsi: u32 = msg.boat_info.get_static_data().mmsi;
        let self_mmsi: u32 = state.boat_info().get_static_data().mmsi;
        
        if boat_mmsi != self_mmsi && IMPLEMENTED_MSGS.binary_search(&msg.message_type).is_ok() {
            if state.boats_registry.is_registered(&boat_mmsi) {
                state.boats_registry.update(msg.boat_info.clone());
            } else {
                state.boats_registry.register(msg.boat_info.clone());
            }

            let slots_map: &SlotsMap = state.slots_map();
            let t_s_owner: Option<u32> = slots_map.slot_owner(t_s);
            let t_s_timeout: Option<u8> = slots_map.slot_timeout(t_s);

            if t_s_owner.is_none() || t_s_owner == Some(self_mmsi) {
                if t_s_timeout.is_some() {
                    slots_map.use_slot(t_s);
                } else {
                    slots_map.mark_slot_as_used(t_s);
                }

                if [1, 2].binary_search(&msg.message_type).is_ok() {
                    let cs_timeout: u8 = msg.communication_state.clone().unwrap().slot_timeout.unwrap();
                    
                    if t_s_owner.is_none() && cs_timeout > 0 {
                        slots_map.book_slot(t_s, self_mmsi, Some(cs_timeout), None);
                    } else if t_s_timeout.is_none() || cs_timeout > 0 {
                        slots_map.slots.write().unwrap()[t_s as usize].timeout = Some(cs_timeout);
                    } else if t_s_timeout == Some(0) || cs_timeout == 0 {
                        slots_map.release_slot(t_s);
                    }

                    if cs_timeout == 0 {
                        let cs_offset: u16 = msg.communication_state.clone().unwrap().slot_offset.unwrap();
                        let rsv_s: u16 = SlotsMap::offseted_slot(t_s, cs_offset);

                        slots_map.book_slot(rsv_s, boat_mmsi, Some(cs_timeout), None);
                        slots_map.release_slot(t_s);
                    }
                } else if msg.message_type == 3 {
                    let cs_keep_flag: bool = msg.communication_state.clone().unwrap().keep_flag.unwrap();
                    let cs_slot_increment: u16 = msg.communication_state.clone().unwrap().slot_increment.unwrap();

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

            log(format!("Message {} reçu du navire {} : {:?}.", msg.message_type, boat_mmsi, msg.boat_info.clone()).blue());
        }
    }


    pub async fn wait_for_slot(state: Arc<AisState>, slot_idx: u16) -> () { // A utiliser dans thread sender !
        let channel: Channel = if slot_idx < SLOTS_PER_MINUTE {Channel::C87B} else {Channel::C88B};
        while SlotsMap::current_slot_number(channel) != slot_idx {
            state.clock_pulse.notified().await;
        }
    }


    pub async fn wait_for_nts(state: Arc<AisState>) -> () {
        let nts: u16 = state.nts();
        let _ = AisRunner::wait_for_slot(Arc::clone(&state), nts).await;
    }


    pub fn set_initial_nss_and_ns(state: Arc<AisState>) -> () {
        let initial_nss_and_ns: u16 = AisRunner::ratdma_slot_selection(Arc::clone(&state), Channel::C87B, 1).unwrap();
        let _ = state.set_ns(initial_nss_and_ns);
        let _ = state.set_nss(initial_nss_and_ns);
    }


    pub fn get_next_ns(state: Arc<AisState>) -> u16 {
        let nss: u16 = state.nss();
        let t_counter: u64 = state.t_counter();
        let ni: u16 = state.ni();
        ((nss as u64 + t_counter * ni as u64) % SLOTS_PER_MINUTE as u64) as u16
    }


    pub fn set_next_ns(state: Arc<AisState>) -> () {
        let next_ns: u16 = AisRunner::get_next_ns(Arc::clone(&state));
        state.set_ns(next_ns);
    }


    pub fn set_next_nts(state: Arc<AisState>) -> u16 {
        let nts: u16 = state.nts();

        let rsv_chn: Channel = if nts < SLOTS_PER_MINUTE { Channel::C88B } else { Channel::C87B };
        let ns: u16 = state.ns();
        let si: u16 = state.si();
        let start_si: u16 = (ns + SLOTS_PER_MINUTE + SLOTS_PER_MINUTE - si.div_euclid(2)) % SLOTS_PER_MINUTE;
        
        let available_nts: Box<[u16]> = state.slots_map().scan_for_free_slots(Some(si), Some(start_si), None, rsv_chn).unwrap();
        let next_nts: u16 = *available_nts.choose(&mut rand::rng()).unwrap();

        let tmo_min: u8 = state.tmo_min();
        let tmo_max: u8 = state.tmo_max();

        let timeout: u8 = rand::rng().random_range(tmo_min..=tmo_max) as u8;

        let mmsi: u32 = state.boat_info().get_static_data().mmsi;
        state.slots_map().book_slot(next_nts, mmsi, Some(timeout), Some(false));
        
        next_nts
    }


    pub fn get_next_nts(state: Arc<AisState>) -> Result<u16, String> {
        let next_ns: u16 = AisRunner::get_next_ns(Arc::clone(&state));
        let si: u16 = state.si();
        let start_si: u16 = (next_ns + SLOTS_PER_MINUTE - si.div_euclid(2)) % SLOTS_PER_MINUTE;

        let available_ss: Box<[u16]> = state.slots_map().scan_for_self_owned_slots(Some(si), Some(start_si), Channel::Any)?;
        
        Ok(*available_ss.choose(&mut rand::rng()).unwrap())
    }


    pub async fn send(state: Arc<AisState>, msg_type: u8, keep_flag: Option<bool>, offset: Option<u16>, slots_nbr: Option<u8>) -> () {
        let ant_tx: &Sender<BitPacker>  = if state.nts() < SLOTS_PER_MINUTE {&state.c_87_b_tx} else {&state.c_88_b_tx};

        let sync_state: u8 = state.sync_state();
        let nts: u16 = state.nts();

        let timeout: Option<u8> = state.slots_map().slot_timeout(nts);
        let recv_stations: u16 = state.recv_stations();

        let com_state: Option<CommunicationState> = if NO_CS_MSGS.binary_search(&msg_type).is_err() {Some(CommunicationState::init(msg_type, sync_state, timeout, offset, Some(nts), Some(recv_stations), offset, slots_nbr, keep_flag).unwrap())} else {None};
        
        let msg: Message = Message::from_info(state.boat_info().as_ref().clone(), msg_type, com_state).unwrap();

        let _ = ant_tx.send(msg.build().unwrap()).await;

        log(format!("Message {} envoyé avec succès sur le slot {}.", msg_type, state.nts()).green());
    }


    pub fn ratdma_slot_selection(state: Arc<AisState>, chn: Channel, lme_rtpri: u8) -> Result<u16, &'static str> {
        let start_s: u16 = SlotsMap::current_slot_number(chn);
        
        let lme_rtes: u16 = SlotsMap::offseted_slot(start_s, 150);

        let slots_range: Box<[u16]> = state.slots_map().slots_idx_range(start_s, lme_rtes, chn);
        let mut candidates: Vec<u16> = Vec::from(state.slots_map().extract_available_slots_idx(slots_range));
    
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


    pub async fn itdma(state: Arc<AisState>, t_s: u16, msg_type: u8, lme_itinc: u16, lme_itsl: u8, lme_itkp: bool) -> () {
        if ITDMA_CS_MSGS.binary_search(&msg_type).is_ok() {
            AisRunner::wait_for_slot(Arc::clone(&state), t_s).await;
            let _ = AisRunner::send(Arc::clone(&state), msg_type, Some(lme_itkp), Some(lme_itinc), Some(lme_itsl)).await;
            state.slots_map().use_slot(t_s);
        } else if NO_CS_MSGS.binary_search(&msg_type).is_ok() {
            AisRunner::wait_for_slot(Arc::clone(&state), t_s).await;
            let _ = AisRunner::send(Arc::clone(&state), msg_type, None, None, None).await;
            state.slots_map().use_slot(t_s);
        }
    }


    pub async fn sotdma_net_entry(state: Arc<AisState>) -> () {
        AisRunner::set_initial_nss_and_ns(Arc::clone(&state));
        let next_nts: u16 = AisRunner::set_next_nts(Arc::clone(&state));
        state.set_nts(next_nts);
        log(format!("Premier NTS réservé : {}", state.nts()).yellow());
        AisRunner::wait_for_nts(Arc::clone(&state)).await;
    }


    pub async fn sotdma_first_frame(state: Arc<AisState>) -> () {
        let mut virtual_offset: Option<u16> = None;
        let ref_nts: u16 = state.nts();
        while virtual_offset.is_none() || virtual_offset != Some(0) {
            AisRunner::set_next_ns(Arc::clone(&state));
            let next_nts: u16 = AisRunner::set_next_nts(Arc::clone(&state));
            let si: u16 = state.si();
            let nts: u16 = state.nts();
            let offset: u16 = SlotsMap::slot_offset(Some(nts), next_nts);
            virtual_offset = if SlotsMap::absolute_slot_distance(Some(next_nts), ref_nts) >= si {Some(offset)} else {Some(0)};
            let t_s: u16 = state.nts();
            let _ = AisRunner::itdma(Arc::clone(&state), t_s, 3, virtual_offset.unwrap(), 1, true).await;
            state.increase_t_counter();

            log(format!("NTS réservé pour le prochain message 3 : {}.", next_nts).yellow());

            if virtual_offset.unwrap() != 0 {
                state.set_nts(next_nts);
            } else {
                state.slots_map().release_slot(next_nts);
                state.set_nts(ref_nts);
                state.decrease_t_counter();
            }
        }
    }


    pub async fn sotdma_continuous(state: Arc<AisState>, msg_type: u8) -> () { // A refactor !
        if NO_CS_MSGS.binary_search(&msg_type).is_ok() {
            AisRunner::wait_for_nts(Arc::clone(&state)).await;
            let _ = AisRunner::send(Arc::clone(&state), msg_type, None, None, None).await;
            let nts: u16 = state.nts();
            state.slots_map().use_slot(nts);
            state.increase_t_counter();
            AisRunner::set_next_ns(Arc::clone(&state));

            match AisRunner::get_next_nts(Arc::clone(&state)) {
                Ok(next_nts) => state.set_nts(next_nts),
                Err(e) => {
                    let next_nts: u16 = AisRunner::set_next_nts(Arc::clone(&state));
                    let nts: u16 = state.nts();
                    let offset: u16 = SlotsMap::slot_offset(Some(next_nts), nts);

                    log(format!("NTS manquant détecté. Réservation du NTS {} pour le remplacer.", next_nts).yellow());

                    AisRunner::wait_for_nts(Arc::clone(&state)).await;
                    let _ = AisRunner::itdma(Arc::clone(&state), nts, 3, offset, 1, true).await;
                    state.set_nts(next_nts);
                }
            }
        } else if SOTDMA_CS_MSGS.binary_search(&msg_type).is_ok() {
            AisRunner::wait_for_nts(Arc::clone(&state)).await;
            let nts: u16 = state.nts();
            if state.slots_map().slot_timeout(nts) == Some(0) {
                let nts_channel: Channel = if nts < SLOTS_PER_MINUTE {Channel::C87B} else {Channel::C88B};
                let ns: u16 = state.ns();
                let si: u16 = state.si();
                let start_si: u16 = (ns + SLOTS_PER_MINUTE - si.div_euclid(2)) % SLOTS_PER_MINUTE;
                let available_nts: Box<[u16]> = state.slots_map().scan_for_free_slots(Some(si), Some(start_si), None, nts_channel).unwrap();

                let new_nts: u16 = *available_nts.choose(&mut rand::rng()).unwrap();

                log(format!("NTS {} arrivé à expiration : remplacement par le slot {} après le prochain message.", nts, new_nts).yellow());

                let offset: u16 = SlotsMap::slot_offset(None, new_nts);
                let _ = AisRunner::send(Arc::clone(&state), msg_type, None, Some(offset), None).await;
                state.slots_map().use_slot(nts);
                state.increase_t_counter();
                AisRunner::set_next_ns(Arc::clone(&state));

                match AisRunner::get_next_nts(Arc::clone(&state)) {
                    Ok(next_nts) => {
                        state.set_nts(next_nts);
                        let tmo_min: u8 = state.tmo_min();
                        let tmo_max: u8 = state.tmo_max();
                        let timeout: u8 = rand::rng().random_range(tmo_min..=tmo_max);
                        let mmsi: u32 = state.boat_info().get_static_data().mmsi;
                        state.slots_map().book_slot(new_nts, mmsi, Some(timeout), None);
                    },
                    Err(e) => {
                        let next_nts: u16 = AisRunner::set_next_nts(Arc::clone(&state));

                        log(format!("NTS manquant détecté. Réservation du NTS {} pour le remplacer.", next_nts).yellow());

                        state.set_nts(next_nts);
                        let tmo_min: u8 = state.tmo_min();
                        let tmo_max: u8 = state.tmo_max();
                        let timeout: u8 = rand::rng().random_range(tmo_min..=tmo_max);
                        let mmsi: u32 = state.boat_info().get_static_data().mmsi;
                        state.slots_map().book_slot(new_nts, mmsi, Some(timeout), None);
                    }
                }
            } else {
                let _ = AisRunner::send(Arc::clone(&state), msg_type, None, None, None).await;
                let nts: u16 = state.nts();
                state.slots_map().use_slot(nts);
                state.increase_t_counter();
                AisRunner::set_next_ns(Arc::clone(&state));

                match AisRunner::get_next_nts(Arc::clone(&state)) {
                    Ok(next_nts) => state.set_nts(next_nts),
                    Err(e) => {
                        let next_nts: u16 = AisRunner::set_next_nts(Arc::clone(&state));
                        let nts: u16 = state.nts();
                        let offset: u16 = SlotsMap::slot_offset(Some(next_nts), nts);

                        log(format!("NTS manquant détecté. Réservation du NTS {} pour le remplacer.", next_nts).yellow());

                        AisRunner::wait_for_nts(Arc::clone(&state)).await;
                        let _ = AisRunner::itdma(Arc::clone(&state), nts, 3, offset, 1, true).await;
                        state.set_nts(next_nts);
                    }
                }
            }
        }
    }


    pub fn sotdma_change_rr(&self) -> () {
        todo!()
    }


    pub async fn sotdma(state: Arc<AisState>) -> () {
        log("Initialisation du SOTDMA...".yellow());
        let _ = tokio::time::sleep(Duration::from_secs(0)).await;
        log("Initialisation du SOTMA terminée.".yellow());

        if state.ri() <= 120 {
            log("Entrée sur le réseau SOTDMA...".yellow());
            AisRunner::sotdma_net_entry(Arc::clone(&state)).await;
            log("Entrée sur le réseau terminée.".yellow());
            log("Début de la première frame SOTMA...".yellow());
            let _ = AisRunner::sotdma_first_frame(Arc::clone(&state)).await;
            log("Fin de la première frame.".yellow());
            log("Lancement de la phase continue SOTDMA.".yellow());

            loop {
                let last_msg5_timestamp: i64 = state.last_msg5_timestamp();
                if last_msg5_timestamp == -1 || get_timestamp(None) - last_msg5_timestamp >= 356 {
                    state.set_last_msg5_timestamp(get_timestamp(None));
                    let _ = AisRunner::sotdma_continuous(Arc::clone(&state), 5).await;
                } else {
                    let _ = AisRunner::sotdma_continuous(Arc::clone(&state), 1).await;
                }
            }
        }
    }


    pub async fn start(self) -> () {
        let mut ais_rx: Receiver<Packet> = self.ais_rx;
        let state_arc: Arc<AisState> = Arc::new(self.state);
        let clock_state_arc: Arc<AisState> = Arc::clone(&state_arc);
        let sotdma_state_arc: Arc<AisState> = Arc::clone(&state_arc);

        tokio::spawn(AisRunner::master_clock(clock_state_arc));

        tokio::spawn(async move {
            loop {
                if let Some(packet) = ais_rx.recv().await {
                    match packet.channel {
                        Channel::C87B => AisRunner::handle_transmission(Arc::clone(&state_arc), packet.message, Channel::C87B),
                        Channel::C88B => AisRunner::handle_transmission(Arc::clone(&state_arc), packet.message, Channel::C88B),
                        _ => todo!()
                    }
                }
            }
        });

        AisRunner::sotdma(sotdma_state_arc).await;
    }
}
