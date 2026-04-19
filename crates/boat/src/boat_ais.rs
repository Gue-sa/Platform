use std::sync::{
    Arc,
    atomic::{AtomicI64, AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering::Relaxed},
};

use shared::{
    ais_message::{CommunicationState, Message},
    bitpacker::BitPacker,
    boat_info::BoatInfo,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::{IMPLEMENTED_MSGS, ITDMA_CS_MSGS, NO_CS_MSGS, SLOTS_PER_MINUTE},
        types::{
            AisError, AisMessageError, AisPacket, AisResult, Channel, ClockError, ClockResult,
        },
        utils::get_timestamp,
    },
    impl_arc_access, impl_atomic_access,
    slots_map::SlotsMap,
};
use tokio::{
    sync::{Mutex, Notify, mpsc::*},
    time::Duration,
};

use colored::*;
use rand::{Rng, seq::IndexedRandom};

use crate::{common::utils::*, systemstate::SystemState};

pub struct BoatAisState {
    c_87_b_tx: Sender<BitPacker>,
    c_88_b_tx: Sender<BitPacker>,

    clock_pulse: Notify,

    boat_info: Arc<BoatInfo>,
    boats_registry: Arc<BoatsInfoRegistry>,
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
    sotdma_t_counter: AtomicU64,
    system_state: Arc<SystemState>,
}

pub struct BoatAisRunner {
    state: BoatAisState,
    ais_rx: Mutex<Receiver<AisPacket>>,
}

impl BoatAisState {
    pub fn init(
        c_87_b_tx: Sender<BitPacker>,
        c_88_b_tx: Sender<BitPacker>,
        boat_info: Arc<BoatInfo>,
        boats_registry: Arc<BoatsInfoRegistry>,
        system_state: Arc<SystemState>,
    ) -> Self {
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
            sotdma_t_counter: AtomicU64::from(1),
            system_state: system_state,
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
    impl_atomic_access!(
        last_msg5_timestamp,
        i64,
        last_msg5_timestamp,
        set_last_msg5_timestamp
    );
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

impl BoatAisRunner {
    pub fn init(
        rx: Receiver<AisPacket>,
        c_87_b_tx: Sender<BitPacker>,
        c_88_b_tx: Sender<BitPacker>,
        boat_info: Arc<BoatInfo>,
        boats_registry: Arc<BoatsInfoRegistry>,
        system_state: Arc<SystemState>,
    ) -> Self {
        Self {
            state: BoatAisState::init(
                c_87_b_tx.clone(),
                c_88_b_tx.clone(),
                boat_info,
                boats_registry,
                system_state,
            ),
            ais_rx: Mutex::new(rx),
        }
    }

    pub async fn boat_ais_master_clock(&self) {
        log("Horloge SOTDMA lancée.".yellow());

        loop {
            let now: Duration = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();

            let total_ns: u64 = now.as_nanos() as u64;

            let current_slot_idx: u64 = (total_ns * 3) / 80_000_000;

            let next_slot_idx: u64 = current_slot_idx + 1;
            let next_slot_start_ns: u64 = (next_slot_idx * 80_000_000) / 3;

            let delay_ns: u64 = next_slot_start_ns.saturating_sub(total_ns);

            tokio::time::sleep(Duration::from_nanos(delay_ns)).await;

            self.state.clock_pulse.notify_waiters();
        }
    }

    pub fn handle_transmission(&self, msg: BitPacker, channel: Channel) -> AisResult<Message> {
        let t_s: u16 = SlotsMap::current_slot_number(channel);
        let msg: Message = Message::from_bits(msg)?;
        let boat_mmsi: u32 = msg.boat_info.get_static_data().mmsi;
        let self_mmsi: u32 = self.state.boat_info().get_static_data().mmsi;

        if boat_mmsi != self_mmsi && IMPLEMENTED_MSGS.binary_search(&msg.message_type).is_ok() {
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

    pub async fn wait_for_slot(&self, slot_idx: u16) -> ClockResult<()> {
        let mut last_slots_distance: u16 = SlotsMap::slot_offset(None, slot_idx);

        let channel: Channel = if slot_idx < SLOTS_PER_MINUTE {
            Channel::C87B
        } else {
            Channel::C88B
        };

        while SlotsMap::current_slot_number(channel) != slot_idx {
            self.state.clock_pulse.notified().await;

            let slot_distance: u16 = SlotsMap::slot_offset(None, slot_idx);

            if slot_distance > last_slots_distance {
                println!(
                    "{} {} {}",
                    SlotsMap::current_slot_number(Channel::C87B),
                    slot_idx,
                    slot_distance
                );

                return Err(ClockError::SlotOvershoot);
            } else {
                last_slots_distance = slot_distance;
            }
        }

        Ok(())
    }

    pub async fn wait_for_nts(&self) -> ClockResult<()> {
        let nts: u16 = self.state.nts();
        self.wait_for_slot(nts).await
    }

    pub fn upcoming_ns(&self) -> u16 {
        let nss: u16 = self.state.nss();
        let t_counter: u64 = self.state.t_counter();
        let ni: u16 = self.state.ni();
        ((nss as u64 + t_counter * ni as u64) % SLOTS_PER_MINUTE as u64) as u16
    }

    pub fn upcoming_nts(&self) -> AisResult<u16> {
        let upcoming_ns: u16 = self.upcoming_ns();
        let si: u16 = self.state.si();
        let start_si: u16 = (upcoming_ns + SLOTS_PER_MINUTE - si.div_euclid(2)) % SLOTS_PER_MINUTE;

        let available_ss: Box<[u16]> = self.state.slots_map().scan_for_self_owned_slots(
            Some(si),
            Some(start_si),
            Channel::Any,
        );

        if available_ss.len() == 0 {
            return Err(AisError::NoOwnedSlot);
        }

        Ok(*available_ss.choose(&mut rand::rng()).unwrap())
    }

    pub fn book_new_nts(&self, ns: u16, keep_nts_channel: bool) -> AisResult<u16> {
        let nts: u16 = self.state.nts();
        let si: u16 = self.state.si();

        let rsv_chn: Channel = if nts < SLOTS_PER_MINUTE && keep_nts_channel {
            Channel::C87B
        } else if nts >= SLOTS_PER_MINUTE && keep_nts_channel {
            Channel::C88B
        } else if nts < SLOTS_PER_MINUTE && !keep_nts_channel {
            Channel::C88B
        } else {
            Channel::C87B
        };

        let mut start_si: u16 =
            (ns + SLOTS_PER_MINUTE + SLOTS_PER_MINUTE - si.div_euclid(2)) % SLOTS_PER_MINUTE;

        if self.state.nts() == u16::MAX {
            start_si += si.div_euclid(2);
        }

        let available_nts: Box<[u16]> =
            self.state
                .slots_map()
                .scan_for_free_slots(Some(si), Some(start_si), None, rsv_chn);

        if available_nts.len() < 4 {
            return Err(AisError::NoValidSlotSelection);
        }

        let next_nts: u16 = *available_nts.choose(&mut rand::rng()).unwrap();

        let tmo_min: u8 = self.state.tmo_min();
        let tmo_max: u8 = self.state.tmo_max();

        let timeout: u8 = rand::rng().random_range(tmo_min..=tmo_max) as u8;

        let mmsi: u32 = self.state.boat_info().get_static_data().mmsi;
        self.state
            .slots_map()
            .book_slot(next_nts, mmsi, Some(timeout), Some(false));

        Ok(next_nts)
    }

    pub async fn send(
        &self,
        msg_type: u8,
        keep_flag: Option<bool>,
        offset: Option<u16>,
        slots_nbr: Option<u8>,
        slot: u16
    ) -> () {
        let ant_tx: &Sender<BitPacker> = if self.state.nts() < SLOTS_PER_MINUTE {
            &self.state.c_87_b_tx
        } else {
            &self.state.c_88_b_tx
        };

        let sync_state: u8 = self.state.sync_state();

        let timeout: Option<u8> = self.state.slots_map().slot_timeout(slot);
        let recv_stations: u16 = self.state.recv_stations();

        let com_state: Option<CommunicationState> = if NO_CS_MSGS.binary_search(&msg_type).is_err()
        {
            Some(CommunicationState::init(
                msg_type,
                sync_state,
                timeout,
                offset,
                Some(slot),
                Some(recv_stations),
                offset,
                slots_nbr,
                keep_flag,
            ))
        } else {
            None
        };

        let msg: Message =
            Message::from_info(self.state.boat_info().as_ref().clone(), msg_type, com_state);

        ant_tx.send(msg.build()).await;

        log(format!(
            "Message {} envoyé avec succès sur le slot {}.",
            msg_type,
            slot
        )
        .green());
    }

    pub fn ratdma_slot_selection(&self, chn: Channel, lme_rtpri: u8) -> Result<u16, &'static str> {
        let start_s: u16 = SlotsMap::current_slot_number(chn);

        let lme_rtes: u16 = SlotsMap::offseted_slot(start_s, 150);

        let slots_range: Box<[u16]> = self
            .state
            .slots_map()
            .slots_idx_range(start_s, lme_rtes, chn);
        let mut candidates: Vec<u16> = Vec::from(
            self.state
                .slots_map()
                .extract_available_slots_idx(slots_range),
        );

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

    pub async fn itdma(
        &self,
        t_s: u16,
        msg_type: u8,
        lme_itinc: u16,
        lme_itsl: u8,
        lme_itkp: bool
    ) -> AisResult<()> {
        if ITDMA_CS_MSGS.binary_search(&msg_type).is_ok() {
            self.wait_for_slot(t_s).await?;
            self.send(msg_type, Some(lme_itkp), Some(lme_itinc), Some(lme_itsl), t_s)
                .await;
            self.state.slots_map().use_slot(t_s);
        } else if NO_CS_MSGS.binary_search(&msg_type).is_ok() {
            self.wait_for_slot(t_s).await?;
            self.send(msg_type, None, None, None, t_s).await;
            self.state.slots_map().use_slot(t_s);
        } else {
            return Err(AisError::AisMessage(AisMessageError::UnknownMessageType));
        }

        Ok(())
    }

    pub async fn sotdma_net_entry(&self) -> AisResult<()> {
        let initial_nss_and_ns: u16 = self.ratdma_slot_selection(Channel::C87B, 1).unwrap();
        self.state.set_ns(initial_nss_and_ns);
        self.state.set_nss(initial_nss_and_ns);

        let next_nts: u16 = self.book_new_nts(initial_nss_and_ns, true)?;
        self.state.set_nts(next_nts);
        log(format!("Premier NTS réservé : {}", self.state.nts()).yellow());
        self.wait_for_nts().await?;

        Ok(())
    }

    pub async fn sotdma_first_frame(&self) -> AisResult<()> {
        let mut virtual_offset: Option<u16> = None;
        let ref_nts: u16 = self.state.nts();
        let si: u16 = self.state.si();

        while virtual_offset.is_none() || virtual_offset != Some(0) {
            let nts: u16 = self.state.nts();
            let next_ns: u16 = self.upcoming_ns();

            let next_nts: u16 = self.book_new_nts(next_ns, false)?;
            let offset: u16 = SlotsMap::slot_offset(Some(nts), next_nts);

            virtual_offset = if SlotsMap::absolute_slot_distance(Some(next_nts), ref_nts) >= si {
                Some(offset)
            } else {
                Some(0)
            };

            self.itdma(nts, 3, virtual_offset.unwrap(), 1, true).await?;

            self.state.increase_t_counter();
            self.state.set_ns(next_ns);

            if virtual_offset.unwrap() != 0 {
                self.state.set_nts(next_nts);

                log(format!("NTS réservé pour le prochain message 3 : {}.", next_nts).yellow());
            } else {
                self.state.slots_map().release_slot(next_nts);

                self.state.set_nts(ref_nts);
            }
        }

        Ok(())
    }

    pub async fn sotdma_continuous(self: Arc<Self>) -> AisResult<()> {
        // Ici, on arrive avec les NS / NTS du message qu'on va envoyer juste après et qu'on doit encore construire
        let nts: u16 = self.state.nts().clone();
        let ns: u16 = self.state.ns().clone();
        let si: u16 = self.state.si();

        let next_ns: u16 = self.upcoming_ns();

        let last_msg5_timestamp: i64 = self.state.last_msg5_timestamp();

        match self.upcoming_nts() {
            Ok(next_nts) => {
                if last_msg5_timestamp == -1 || get_timestamp(None) - last_msg5_timestamp >= 356 {
                    let msg5_slot: u16 = self.book_new_nts(next_ns, false)?;

                    self.state.slots_map().release_slot(msg5_slot);

                    let offset: u16 = SlotsMap::slot_offset(Some(nts), msg5_slot);

                    log(format!(
                        "Réservation du slot {} pour émettre le prochain message 5.",
                        msg5_slot
                    )
                    .yellow());

                    self.wait_for_nts().await?;

                    self.itdma(nts, 3, offset, 1, true).await?;

                    self.state.increase_t_counter();
                    self.state.set_ns(next_ns);
                    self.state.set_nts(next_nts);

                    self.state.set_last_msg5_timestamp(get_timestamp(None));

                    tokio::spawn(async move {
                        self.wait_for_slot(msg5_slot).await;
                        self.send(5, None, None, None, msg5_slot).await;
                    });
                } else if self.state.slots_map().slot_timeout(nts) == Some(0) {
                    let new_nts: u16 = self.book_new_nts(ns, true)?;

                    log(format!("NTS {} arrivé à expiration : remplacement par le slot {} après le prochain message.", nts, new_nts).yellow());

                    let offset: u16 = SlotsMap::slot_offset(Some(nts), new_nts);

                    self.wait_for_nts().await?;

                    self.send(1, None, Some(offset), None, nts).await;
                    self.state.slots_map().use_slot(nts);

                    self.state.increase_t_counter();
                    self.state.set_ns(next_ns);
                    self.state.set_nts(next_nts);
                } else {
                    self.wait_for_nts().await?;

                    self.send(1, None, None, None, nts).await;
                    self.state.slots_map().use_slot(nts);

                    self.state.increase_t_counter();
                    self.state.set_ns(next_ns);
                    self.state.set_nts(next_nts);
                }
            }
            Err(_) => {
                let new_nts: u16 = self.book_new_nts(next_ns, false)?;
                let offset: u16 = SlotsMap::slot_offset(Some(nts), new_nts);

                log(format!(
                    "NTS manquant détecté. Réservation du NTS {} pour le remplacer.",
                    new_nts
                )
                .yellow());

                self.wait_for_nts().await?;

                self.itdma(nts, 3, offset, 1, true).await?;

                self.state.increase_t_counter();
                self.state.set_ns(next_ns);
                self.state.set_nts(new_nts);
            }
        }

        Ok(())
    }

    pub fn sotdma_change_rr(&self) -> () {
        todo!()
    }

    pub async fn sotdma(self: Arc<Self>) -> AisResult<()> {
        log("Initialisation du SOTDMA...".yellow());

        tokio::time::sleep(Duration::from_secs(0)).await;

        log("Initialisation du SOTMA terminée.".yellow());

        if self.state.ri() <= 120 {
            log("Entrée sur le réseau SOTDMA...".yellow());

            match self.sotdma_net_entry().await {
                Ok(_) => {}
                Err(_) => return Err(AisError::SotdmaInitFailed),
            }

            log("Entrée sur le réseau terminée.".yellow());
            log("Début de la première frame SOTMA...".yellow());

            match self.sotdma_first_frame().await {
                Ok(_) => {}
                Err(e) => {
                    println!("Erreur : {:?}", e);
                    return Err(AisError::SotdmaInitFailed);
                }
            }

            log("Fin de la première frame.".yellow());
            log("Lancement de la phase continue SOTDMA.".yellow());

            loop {
                match self.clone().sotdma_continuous().await {
                    Ok(_) => {}
                    Err(_) => {
                        self.state.increase_t_counter();
                        self.state.set_ns(self.upcoming_ns());
                        self.state.set_nts(self.upcoming_nts()?);

                        log("L'AIS a subi une erreur qui a rendu l'émission du message initialement prévu impossible. Il continuera probablement à fonctionner normalement, mais il est préférable de surveiller son bon comportement pour une durée d'une minute révolue.".red());
                    }
                }
            }
        } else {
            return Err(AisError::SotdmaInitFailed);
        }

        Ok(())
    }

    pub async fn start(self) -> () {
        let runner_arc: Arc<Self> = Arc::new(self);
        let c87b_runner_arc: Arc<Self> = runner_arc.clone();
        let c88b_runner_arc: Arc<Self> = runner_arc.clone();
        let sotdma_runner_arc: Arc<Self> = runner_arc.clone();
        let clock_runner_arc: Arc<Self> = runner_arc.clone();

        tokio::spawn(async move {
            clock_runner_arc.clone().boat_ais_master_clock().await;
        });

        tokio::spawn(async move {
            loop {
                if let Some(packet) = runner_arc.ais_rx.lock().await.recv().await {
                    match packet.channel {
                        Channel::C87B => {
                            match c87b_runner_arc.handle_transmission(packet.message, Channel::C87B)
                            {
                                Ok(msg) => {
                                    log(format!(
                                        "Message {} reçu du navire {} : {:#?}.",
                                        msg.message_type,
                                        msg.boat_info.get_static_data().mmsi,
                                        msg.boat_info.clone()
                                    )
                                    .blue());
                                }
                                Err(e) => match e {
                                    AisError::SelfEmittedMessage => {}
                                    _ => {
                                        log("Message corrompu reçu et ignoré.".red());
                                    }
                                },
                            }
                        }
                        Channel::C88B => {
                            match c88b_runner_arc.handle_transmission(packet.message, Channel::C88B)
                            {
                                Ok(msg) => {
                                    log(format!(
                                        "Message {} reçu du navire {} : {:#?}.",
                                        msg.message_type,
                                        msg.boat_info.get_static_data().mmsi,
                                        msg.boat_info.clone()
                                    )
                                    .blue());
                                }
                                Err(e) => match e {
                                    AisError::SelfEmittedMessage => {}
                                    _ => {
                                        log("Message corrompu reçu et ignoré.".red());
                                    }
                                },
                            }
                        }
                        _ => todo!(),
                    }
                }
            }
        });

        match sotdma_runner_arc.sotdma().await {
            Ok(_) => {}
            Err(_) => {
                panic!(
                    "Le SOTDMA a subi une erreur irrécupérable. Veuillez redémarrer l'AIS manuellement."
                )
            }
        }
    }
}
