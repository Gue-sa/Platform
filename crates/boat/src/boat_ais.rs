use crate::systemstate::SystemState;
use colored::*;
use core::panic;
use rand::{RngExt, seq::IndexedRandom};
use shared::{
    ais_message::{AisMessage, CommunicationState},
    bitpacker::BitPacker,
    boat_info::BoatInfo,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::{IMPLEMENTED_MSGS, ITDMA_CS_MSGS, NO_CS_MSGS, SLOTS_PER_MINUTE},
        errors::{AisError, AisMessageError, AisResult, ClockError, ClockResult},
        types::{AisMessageType, AisPacket, Channel, LogEvent},
        utils::get_timestamp,
    },
    impl_arc_access, impl_atomic_access,
    slots_map::SlotsMap,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering::Relaxed},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{Mutex, Notify, mpsc::*},
    task::JoinHandle,
    time::{Duration, sleep},
};

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
    logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
}

impl BoatAisState {
    pub fn init(
        c_87_b_tx: Sender<BitPacker>,
        c_88_b_tx: Sender<BitPacker>,
        boat_info: Arc<BoatInfo>,
        boats_registry: Arc<BoatsInfoRegistry>,
        system_state: Arc<SystemState>,
    ) -> AisResult<Self> {
        let mmsi = *boat_info.get_static_data()?.mmsi();

        let sotdma_ri = 10;
        let sotdma_rr = 60. / sotdma_ri as f64;
        let sotdma_ni = (SLOTS_PER_MINUTE as f64 / sotdma_rr).round() as u16;
        let sotdma_si = (0.2 * sotdma_ni as f64).round() as u16;

        Ok(Self {
            c_87_b_tx: c_87_b_tx,
            c_88_b_tx: c_88_b_tx,
            clock_pulse: Notify::new(),
            boat_info: boat_info,
            boats_registry: boats_registry,
            slots_map: SlotsMap::new(mmsi),
            recv_stations: AtomicU16::new(0),
            sync_state: AtomicU8::new(0),
            last_msg5_timestamp: AtomicI64::new(-1),
            sotdma_nss: AtomicU16::new(u16::MAX),
            sotdma_ns: AtomicU16::new(u16::MAX),
            sotdma_nts: AtomicU16::new(u16::MAX),
            sotdma_ri: AtomicU32::from(sotdma_ri),
            sotdma_rr: sotdma_rr,
            sotdma_ni: AtomicU16::from(sotdma_ni),
            sotdma_si: AtomicU16::from(sotdma_si),
            sotdma_tmo_min: AtomicU8::from(3),
            sotdma_tmo_max: AtomicU8::from(7),
            sotdma_t_counter: AtomicU64::from(1),
            system_state: system_state,
        })
    }

    pub fn slots_map(&self) -> &SlotsMap {
        &self.slots_map
    }

    fn increase_t_counter(&self) -> () {
        self.sotdma_t_counter.fetch_add(1, Relaxed);
    }

    fn decrease_t_counter(&self) -> () {
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
        c87b_tx: Sender<BitPacker>,
        c88b_tx: Sender<BitPacker>,
        boat_info: Arc<BoatInfo>,
        boats_reg: Arc<BoatsInfoRegistry>,
        sys_state: Arc<SystemState>,
        logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
    ) -> AisResult<Self> {
        Ok(Self {
            state: BoatAisState::init(
                c87b_tx.clone(),
                c88b_tx.clone(),
                boat_info,
                boats_reg,
                sys_state,
            )?,
            ais_rx: Mutex::new(rx),
            logs_cli_tx: logs_cli_tx,
        })
    }

    fn logs_cli_tx(&self) -> std::sync::mpsc::Sender<LogEvent> {
        self.logs_cli_tx.clone()
    }

    async fn run_boat_ais_master_clock(&self) -> ClockResult<()> {
        self.logs_cli_tx()
            .send(LogEvent::System("Lancement de l'horloge AIS...".yellow()));

        loop {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?;

            let total_ns = now.as_nanos() as u64;

            let current_si = (total_ns * 3) / 80_000_000;

            let next_si = current_si + 1;
            let next_slot_start_ns = (next_si * 80_000_000) / 3;

            let delay_ns = next_slot_start_ns.saturating_sub(total_ns);

            sleep(Duration::from_nanos(delay_ns)).await;

            self.state.clock_pulse.notify_waiters();
        }
    }

    fn handle_transmission(&self, msg: &BitPacker, channel: Channel) -> AisResult<AisMessage> {
        let t_si = SlotsMap::current_si(channel);
        let msg = AisMessage::from_bits(msg)?;
        let boat_mmsi = *msg.boat_info().get_static_data()?.mmsi();
        let self_mmsi = *self.state.boat_info().get_static_data()?.mmsi();

        if boat_mmsi != self_mmsi && IMPLEMENTED_MSGS.binary_search(msg.message_type()).is_ok() {
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

                match msg.message_type() {
                    AisMessageType::Msg1 | AisMessageType::Msg2 => {
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
                    }
                    AisMessageType::Msg3 => {
                        let com_state_keep_flag = *msg.communication_state()?.keep_flag()?;
                        let com_state_slot_increment =
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
                    _ => {}
                }
            }
        } else {
            return Err(AisError::SelfEmittedMessage);
        }

        Ok(msg)
    }

    async fn wait_for_slot(&self, slot_idx: u16) -> ClockResult<()> {
        let mut last_si_distance = SlotsMap::si_offset(None, slot_idx);

        let channel = if slot_idx < SLOTS_PER_MINUTE {
            Channel::C87B
        } else {
            Channel::C88B
        };

        while SlotsMap::current_si(channel) != slot_idx {
            self.state.clock_pulse.notified().await;

            let slot_distance = SlotsMap::si_offset(None, slot_idx);

            if slot_distance > last_si_distance {
                return Err(ClockError::SlotOvershoot);
            } else {
                last_si_distance = slot_distance;
            }
        }

        Ok(())
    }

    async fn wait_for_nts(&self) -> ClockResult<()> {
        let nts = self.state.nts();
        self.wait_for_slot(nts).await
    }

    fn upcoming_ns(&self) -> u16 {
        let nss = self.state.nss();
        let t_counter = self.state.t_counter();
        let ni = self.state.ni();
        ((nss as u64 + t_counter * ni as u64) % SLOTS_PER_MINUTE as u64) as u16
    }

    fn upcoming_nts(&self) -> AisResult<u16> {
        let upcoming_ns = self.upcoming_ns();
        let si = self.state.si();
        let start_si = (upcoming_ns + SLOTS_PER_MINUTE - si.div_euclid(2)) % SLOTS_PER_MINUTE;

        let available_ss = self.state.slots_map().scan_for_self_owned_ssi(
            Some(si),
            Some(start_si),
            Channel::Any,
        )?;

        Ok(*available_ss
            .choose(&mut rand::rng())
            .ok_or(AisError::NoOwnedSlot)?)
    }

    fn book_new_nts(&self, ns: u16, keep_nts_channel: bool) -> AisResult<u16> {
        let nts = self.state.nts();
        let si = self.state.si();

        let rsv_chn = if nts < SLOTS_PER_MINUTE && keep_nts_channel {
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

        let available_nts =
            self.state
                .slots_map()
                .scan_for_free_ssi(Some(si), Some(start_si), None, rsv_chn)?;

        if available_nts.len() < 4 {
            return Err(AisError::NoValidSlotSelection);
        }

        let next_nts = *available_nts
            .choose(&mut rand::rng())
            .ok_or(AisError::NoValidSlotSelection)?;

        let tmo_min = self.state.tmo_min();
        let tmo_max = self.state.tmo_max();

        let timeout = rand::rng().random_range(tmo_min..=tmo_max) as u8;

        let mmsi = *self.state.boat_info().get_static_data()?.mmsi();
        self.state
            .slots_map()
            .book_slot(next_nts, mmsi, Some(timeout), Some(false))?;

        Ok(next_nts)
    }

    async fn send(
        &self,
        msg_type: AisMessageType,
        keep_flag: Option<bool>,
        offset: Option<u16>,
        slots_nbr: Option<u8>,
        t_si: u16,
    ) -> AisResult<()> {
        let ant_tx = if self.state.nts() < SLOTS_PER_MINUTE {
            &self.state.c_87_b_tx
        } else {
            &self.state.c_88_b_tx
        };

        let sync_state = self.state.sync_state();

        let timeout = self.state.slots_map().slot_timeout(t_si)?;
        let recv_stations = self.state.recv_stations();

        let com_state = if NO_CS_MSGS.binary_search(&msg_type).is_err() {
            Some(CommunicationState::init(
                msg_type,
                sync_state,
                timeout,
                offset,
                Some(t_si),
                Some(recv_stations),
                offset,
                slots_nbr,
                keep_flag,
            ))
        } else {
            None
        };

        let msg = AisMessage::from_info(self.state.boat_info().as_ref(), msg_type, com_state)?;

        ant_tx.send(msg.build()?).await?;

        self.logs_cli_tx().send(LogEvent::Ais(
            format!(
                "Message {} envoyé avec succès sur le slot {}.",
                msg_type as u8, t_si
            )
            .green(),
        ));

        Ok(())
    }

    fn ratdma_slot_selection(&self, chn: Channel, lme_rtpri: u8) -> AisResult<u16> {
        let start_si = SlotsMap::current_si(chn);

        let lme_rtes = SlotsMap::offseted_si(start_si, 150);

        let slots_range = self.state.slots_map().ssi_range(start_si, lme_rtes, chn);
        let mut candidates: Vec<u16> =
            Vec::from(self.state.slots_map().filter_available_ssi(&slots_range)?);

        let mut candidate = *candidates
            .choose(&mut rand::rng())
            .ok_or(AisError::NoValidSlotSelection)?;

        let mut lme_rtcsc = candidates.len() as u8;
        let mut lme_rta = 0;

        let lme_rtps = 100. / lme_rtcsc as f64;
        let lme_rtp1 = rand::rng().random_range(0.0..=100.0);
        let mut lme_rtp2 = lme_rtps;
        let mut lme_rtpi = (100. - lme_rtp2) / lme_rtcsc as f64;

        while lme_rtp1 > lme_rtp2 as f64 {
            lme_rtp2 += lme_rtpi;
            lme_rtcsc -= 1;
            lme_rta += 1;
            lme_rtpi = (100. - lme_rtp2) / lme_rtcsc as f64;
            candidates.retain(|c| *c != candidate);
            candidate = *candidates
                .choose(&mut rand::rng())
                .ok_or(AisError::NoValidSlotSelection)?;
        }

        Ok(candidate)
    }

    async fn itdma(
        &self,
        t_s: u16,
        msg_type: AisMessageType,
        lme_itinc: u16,
        lme_itsl: u8,
        lme_itkp: bool,
    ) -> AisResult<()> {
        if ITDMA_CS_MSGS.binary_search(&msg_type).is_ok() {
            self.wait_for_slot(t_s).await?;
            self.send(
                msg_type,
                Some(lme_itkp),
                Some(lme_itinc),
                Some(lme_itsl),
                t_s,
            )
            .await?;
            self.state.slots_map().use_slot(t_s)?;
        } else if NO_CS_MSGS.binary_search(&msg_type).is_ok() {
            self.wait_for_slot(t_s).await?;
            self.send(msg_type, None, None, None, t_s).await?;
            self.state.slots_map().use_slot(t_s)?;
        } else {
            return Err(AisError::AisMessage(
                AisMessageError::MessageTypeNotImplemented,
            ));
        }

        Ok(())
    }

    async fn sotdma_net_entry(&self) -> AisResult<()> {
        let initial_nss_and_ns = self.ratdma_slot_selection(Channel::C87B, 1)?;
        self.state.set_ns(initial_nss_and_ns);
        self.state.set_nss(initial_nss_and_ns);

        let next_nts = self.book_new_nts(initial_nss_and_ns, true)?;
        self.state.set_nts(next_nts);
        self.logs_cli_tx().send(LogEvent::Ais(
            format!("Premier NTS réservé : {}", self.state.nts()).yellow(),
        ));
        self.wait_for_nts().await?;

        Ok(())
    }

    async fn sotdma_first_frame(&self) -> AisResult<()> {
        let mut virtual_offset = u16::MAX;
        let ref_nts = self.state.nts();
        let si = self.state.si();

        while virtual_offset == u16::MAX || virtual_offset != 0 {
            let nts = self.state.nts();
            let next_ns = self.upcoming_ns();

            let next_nts = self.book_new_nts(next_ns, false)?;
            let offset = SlotsMap::si_offset(Some(nts), next_nts);

            virtual_offset = if SlotsMap::absolute_si_distance(Some(next_nts), ref_nts) >= si {
                offset
            } else {
                0
            };

            self.itdma(nts, AisMessageType::Msg3, virtual_offset, 1, true)
                .await?;

            self.state.increase_t_counter();
            self.state.set_ns(next_ns);

            if virtual_offset != 0 {
                self.state.set_nts(next_nts);

                self.logs_cli_tx().send(LogEvent::Ais(
                    format!("NTS réservé pour le prochain message 3 : {}.", next_nts).yellow(),
                ));
            } else {
                self.state.slots_map().release_slot(next_nts)?;

                self.state.set_nts(ref_nts);
            }
        }

        Ok(())
    }

    async fn sotdma_continuous(self: Arc<Self>) -> AisResult<()> {
        // Ici, on arrive avec les NS / NTS du message qu'on va envoyer juste après et qu'on doit encore construire
        let nts = self.state.nts();
        let ns = self.state.ns();

        let next_ns = self.upcoming_ns();

        let last_msg5_timestamp = self.state.last_msg5_timestamp();

        match self.upcoming_nts() {
            Ok(next_nts) => {
                if last_msg5_timestamp == -1 || get_timestamp(None) - last_msg5_timestamp >= 356 {
                    let msg5_slot = self.book_new_nts(next_ns, false)?;

                    self.state.slots_map().release_slot(msg5_slot)?;

                    let offset = SlotsMap::si_offset(Some(nts), msg5_slot);

                    self.logs_cli_tx().send(LogEvent::Ais(
                        format!(
                            "Réservation du slot {} pour émettre le prochain message 5.",
                            msg5_slot
                        )
                        .yellow(),
                    ));

                    self.wait_for_nts().await?;

                    self.itdma(nts, AisMessageType::Msg3, offset, 1, true)
                        .await?;

                    self.state.increase_t_counter();
                    self.state.set_ns(next_ns);
                    self.state.set_nts(next_nts);

                    self.state.set_last_msg5_timestamp(get_timestamp(None));

                    tokio::spawn(async move {
                        if let Ok(_) = self.wait_for_slot(msg5_slot).await {
                            let _ = self
                                .send(AisMessageType::Msg5, None, None, None, msg5_slot)
                                .await;
                        }
                    });
                } else if self.state.slots_map().slot_timeout(nts)? == Some(0) {
                    let new_nts = self.book_new_nts(ns, true)?;

                    self.logs_cli_tx().send(LogEvent::Ais(format!("NTS {} arrivé à expiration : remplacement par le slot {} après le prochain message.", nts, new_nts).yellow()));

                    let offset = SlotsMap::si_offset(Some(nts), new_nts);

                    self.wait_for_nts().await?;

                    self.send(AisMessageType::Msg1, None, Some(offset), None, nts)
                        .await?;
                    self.state.slots_map().use_slot(nts)?;

                    self.state.increase_t_counter();
                    self.state.set_ns(next_ns);
                    self.state.set_nts(next_nts);
                } else {
                    self.wait_for_nts().await?;

                    self.send(AisMessageType::Msg1, None, None, None, nts)
                        .await?;
                    self.state.slots_map().use_slot(nts)?;

                    self.state.increase_t_counter();
                    self.state.set_ns(next_ns);
                    self.state.set_nts(next_nts);
                }
            }
            Err(_) => {
                let new_nts = self.book_new_nts(next_ns, false)?;
                let offset = SlotsMap::si_offset(Some(nts), new_nts);

                self.logs_cli_tx().send(LogEvent::Ais(
                    format!(
                        "NTS manquant détecté. Réservation du NTS {} pour le remplacer.",
                        new_nts
                    )
                    .yellow(),
                ));

                self.wait_for_nts().await?;

                self.itdma(nts, AisMessageType::Msg3, offset, 1, true)
                    .await?;

                self.state.increase_t_counter();
                self.state.set_ns(next_ns);
                self.state.set_nts(new_nts);
            }
        }

        Ok(())
    }

    fn sotdma_change_rr(&self) -> () {
        todo!()
    }

    async fn run_sotdma(self: Arc<Self>) -> AisResult<()> {
        self.logs_cli_tx()
            .send(LogEvent::System("Initialisation du SOTDMA...".yellow()));

        sleep(Duration::from_secs(0)).await;

        if self.state.ri() <= 120 {
            self.logs_cli_tx()
                .send(LogEvent::System("Entrée sur le réseau SOTDMA...".yellow()));

            match self.sotdma_net_entry().await {
                Ok(_) => {}
                Err(_) => return Err(AisError::SotdmaInitFailed),
            }

            self.logs_cli_tx()
                .send(LogEvent::System("Entrée sur le réseau terminée.".yellow()));
            self.logs_cli_tx().send(LogEvent::System(
                "Début de la première frame SOTMA...".yellow(),
            ));

            match self.sotdma_first_frame().await {
                Ok(_) => {}
                Err(e) => {
                    return Err(AisError::SotdmaInitFailed);
                }
            }

            self.logs_cli_tx()
                .send(LogEvent::System("Fin de la première frame.".yellow()));
            self.logs_cli_tx().send(LogEvent::System(
                "Lancement de la phase continue SOTDMA.".yellow(),
            ));

            loop {
                match self.clone().sotdma_continuous().await {
                    Ok(_) => {}
                    Err(_) => {
                        self.state.increase_t_counter();
                        self.state.set_ns(self.upcoming_ns());
                        self.state.set_nts(self.upcoming_nts()?);

                        self.logs_cli_tx().send(LogEvent::Ais("L'AIS a subi une erreur qui a rendu l'émission du message initialement prévu impossible. Il continuera probablement à fonctionner normalement, mais il est préférable de surveiller son bon comportement pour une durée d'une minute révolue.".red()));
                    }
                }
            }
        } else {
            return Err(AisError::SotdmaInitFailed);
        }
    }

    async fn run_listeners(&self) -> () {
        self.logs_cli_tx().send(LogEvent::System(
            "Lancement de l'écoute de l'AIS...".yellow(),
        ));

        loop {
            let pck_opt = {
                let mut rx = self.ais_rx.lock().await;
                rx.recv().await
            };

            if let Some(pck) = pck_opt {
                match pck.channel {
                    Channel::C87B => match self.handle_transmission(&pck.message, Channel::C87B) {
                        Ok(msg) => {
                            self.logs_cli_tx().send(LogEvent::Ais(
                                format!(
                                    "Message {} reçu du navire {} : {:#?}.",
                                    *msg.message_type() as u8,
                                    *msg.boat_info().get_static_data().unwrap().mmsi(),
                                    msg.boat_info()
                                )
                                .blue(),
                            ));
                        }
                        Err(e) => match e {
                            AisError::SelfEmittedMessage => {}
                            _ => {
                                self.logs_cli_tx()
                                    .send(LogEvent::Ais("Message corrompu reçu et ignoré.".red()));
                            }
                        },
                    },
                    Channel::C88B => match self.handle_transmission(&pck.message, Channel::C88B) {
                        Ok(msg) => {
                            self.logs_cli_tx().send(LogEvent::Ais(
                                format!(
                                    "Message {} reçu du navire {} : {:#?}.",
                                    *msg.message_type() as u8,
                                    *msg.boat_info().get_static_data().unwrap().mmsi(),
                                    msg.boat_info()
                                )
                                .blue(),
                            ));
                        }
                        Err(e) => match e {
                            AisError::SelfEmittedMessage => {}
                            _ => {
                                self.logs_cli_tx()
                                    .send(LogEvent::Ais("Message corrompu reçu et ignoré.".red()));
                            }
                        },
                    },
                    _ => todo!(),
                }
            }
        }
    }

    pub fn start(
        self,
    ) -> (
        JoinHandle<()>,
        JoinHandle<()>,
        JoinHandle<()>,
        JoinHandle<()>,
    ) {
        let runner_arc = Arc::new(self);
        let slots_map_cleanup_runner_arc = runner_arc.clone();
        let listeners_runner_arc = runner_arc.clone();
        let sotdma_runner_arc = runner_arc.clone();
        let clock_runner_arc = runner_arc.clone();

        runner_arc.logs_cli_tx().send(LogEvent::System(
            "Lancement du nettoyage automatique de la table des slots AIS...".yellow(),
        ));

        (
            tokio::spawn(async move {
                let _ = slots_map_cleanup_runner_arc
                    .state
                    .slots_map()
                    .run_cleanup_task()
                    .await;
                slots_map_cleanup_runner_arc.logs_cli_tx().send(LogEvent::System("Le daemon de nettoyage des slots s'est arrêté de façon inattendue. Veuillez redémarrer l'AIS manuellement.".red()));
                panic!();
            }),
            tokio::spawn(async move {
                let _ = clock_runner_arc.clone().run_boat_ais_master_clock().await;
                clock_runner_arc.logs_cli_tx().send(LogEvent::System("L'horloge AIS s'est arrêtée de façon inattendue. Veuillez redémarrer l'AIS manuellement.".red()));
                panic!();
            }),
            tokio::spawn(async move {
                listeners_runner_arc.run_listeners().await;
                listeners_runner_arc.logs_cli_tx().send(LogEvent::System("L'écoute AIS s'est arrêtée de façon inattendue. Veuillez redémarrer l'AIS manuellement.".red()));
                panic!();
            }),
            tokio::spawn(async move {
                let _ = sotdma_runner_arc.clone().run_sotdma().await;
                sotdma_runner_arc.logs_cli_tx().send(LogEvent::System("Le SOTDMA s'est arrêté de façon inattendue. Veuillez redémarrer l'AIS manuellement.".red()));
                panic!();
            }),
        )
    }
}
