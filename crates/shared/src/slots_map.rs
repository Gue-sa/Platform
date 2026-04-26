use crate::{
    common::{
        constants::SLOTS_PER_MINUTE,
        types::Channel,
        utils::{dt_to_slots_idx, get_current_dt},
    },
    slot::Slot,
};
use getset::MutGetters;
use rand::seq::IndexedRandom;
use std::{
    array,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::time::{Duration, Instant, interval_at};

#[derive(Debug, Clone, MutGetters)]
pub struct SlotsMap {
    #[getset(get_mut = "pub")]
    slots: Arc<RwLock<[Slot; 2 * SLOTS_PER_MINUTE as usize]>>,
    boat_mmsi: u32,
}

impl SlotsMap {
    pub fn new(boat_mmsi: u32) -> Self {
        Self {
            slots: Arc::new(RwLock::new(array::from_fn(|i: usize| Slot::new(i as u16)))),
            boat_mmsi: boat_mmsi,
        }
    }

    pub async fn run_cleanup_task(&self) -> () {
        loop {
            for slot in self.slots.write().unwrap().iter_mut() {
                match *slot.frames_since_last_use() {
                    -2 => {
                        if !slot.is_free() {
                            slot.release();
                        }
                    }
                    3 => slot.release(),
                    _ => {
                        slot.set_frames_since_last_use(*slot.frames_since_last_use() + 1);
                    }
                }
            }

            let ns_since_epoch: u128 = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let ns_until_next_min: u128 = 60_000_000_000 - (ns_since_epoch % 60_000_000_000);

            let start_instant: Instant =
                Instant::now() + Duration::from_nanos(ns_until_next_min as u64);

            interval_at(start_instant, Duration::from_secs(60))
                .tick()
                .await;
        }
    }

    pub fn use_slot(&self, si: u16) -> () {
        self.slots.write().unwrap()[si as usize].tick();
    }

    pub fn flag_slot_as_used(&self, si: u16) -> () {
        self.slots.write().unwrap()[si as usize].flag_as_used();
    }

    pub fn slot_owner(&self, si: u16) -> Option<u32> {
        *self.slots.read().unwrap()[si as usize].owner()
    }

    pub fn slot_timeout(&self, si: u16) -> Option<u8> {
        *self.slots.read().unwrap()[si as usize].timeout()
    }

    pub fn slot_channel(&self, si: u16) -> Channel {
        *self.slots.read().unwrap()[si as usize].channel()
    }

    pub fn is_slot_free(&self, si: u16) -> bool {
        self.slots.read().unwrap()[si as usize].is_free()
    }

    pub fn is_slot_expired(&self, si: u16) -> bool {
        *self.slots.read().unwrap()[si as usize].frames_since_last_use() > 2
    }

    pub fn is_slot_current(&self, si: u16) -> bool {
        dt_to_slots_idx(None).contains(&si)
    }

    pub fn set_slot_timeout(&self, si: u16, timeout: Option<u8>) {
        self.slots.write().unwrap()[si as usize].set_timeout(timeout);
    }

    pub fn book_slot(
        &self,
        si: u16,
        mmsi: u32,
        timeout: Option<u8>,
        is_assigned: Option<bool>,
    ) -> () {
        self.slots.write().unwrap()[si as usize].book(mmsi, timeout, is_assigned.unwrap_or(false));
    }

    pub fn release_slot(&self, si: u16) -> () {
        self.slots.write().unwrap()[si as usize].release();
    }

    pub fn current_si(chn: Channel) -> u16 {
        let current_dt: chrono::DateTime<chrono::Local> = get_current_dt();
        match chn {
            Channel::C87B => dt_to_slots_idx(Some(current_dt))[0],
            Channel::C88B => dt_to_slots_idx(Some(current_dt))[1],
            _ => dt_to_slots_idx(Some(current_dt))[0],
        }
    }

    pub fn si_offset(s0: Option<u16>, s1: u16) -> u16 {
        let s0: u16 = s0.unwrap_or(SlotsMap::current_si(Channel::C87B));

        (s1 % SLOTS_PER_MINUTE + SLOTS_PER_MINUTE - s0 % SLOTS_PER_MINUTE) % SLOTS_PER_MINUTE
    }

    pub fn absolute_si_distance(s0: Option<u16>, s1: u16) -> u16 {
        let s0: u16 = s0.unwrap_or(SlotsMap::current_si(Channel::C87B));

        (s0 % SLOTS_PER_MINUTE).abs_diff(s1 % SLOTS_PER_MINUTE)
    }

    pub fn offseted_si(si: u16, offset: u16) -> u16 {
        let offseted_si: u16 = (si + offset) % SLOTS_PER_MINUTE;

        if si < SLOTS_PER_MINUTE {
            offseted_si
        } else {
            offseted_si + SLOTS_PER_MINUTE
        }
    }

    pub fn ssi_range(&self, start_si: u16, end_si: u16, chn: Channel) -> Box<[u16]> {
        // Prend en argument les slots % SLOTS_PER_MINUTE ! L'ajustement se fait tout seul en fonction de channel !
        if start_si <= end_si {
            match chn {
                Channel::C87B => (start_si..=end_si).collect(),
                Channel::C88B => {
                    (start_si + SLOTS_PER_MINUTE..=end_si + SLOTS_PER_MINUTE).collect()
                }
                Channel::Any => [
                    self.ssi_range(start_si, end_si, Channel::C87B),
                    self.ssi_range(start_si, end_si, Channel::C88B),
                ]
                .concat()
                .into_boxed_slice(),
                _ => Box::new([]),
            }
        } else {
            match chn {
                Channel::C87B => (start_si..SLOTS_PER_MINUTE).chain(0..=end_si).collect(),
                Channel::C88B => (start_si..2 * SLOTS_PER_MINUTE)
                    .chain(SLOTS_PER_MINUTE..=end_si)
                    .collect(),
                Channel::Any => [
                    self.ssi_range(start_si, end_si, Channel::C87B),
                    self.ssi_range(start_si, end_si, Channel::C88B),
                ]
                .concat()
                .into_boxed_slice(),
                _ => Box::new([]),
            }
        }
    }

    fn get_available_ssi(&self, chn: Option<Channel>) -> Box<[u16]> {
        let chn: Channel = chn.unwrap_or(Channel::Any);

        match chn {
            Channel::Any => (0..2 * SLOTS_PER_MINUTE)
                .filter(|si: &u16| self.is_slot_free(*si))
                .collect(),
            Channel::C87B => (0..2 * SLOTS_PER_MINUTE)
                .filter(|si: &u16| {
                    self.is_slot_free(*si) && matches!(self.slot_channel(*si), Channel::C87B)
                })
                .collect(),
            Channel::C88B => (0..2 * SLOTS_PER_MINUTE)
                .filter(|si: &u16| {
                    self.is_slot_free(*si) && matches!(self.slot_channel(*si), Channel::C88B)
                })
                .collect(),
            _ => Box::new([]),
        }
    }

    pub fn filter_unavailable_ssi(&self, slots: Box<[u16]>) -> Box<[u16]> {
        slots
            .iter()
            .filter(|slot_number: &&u16| self.is_slot_free(**slot_number))
            .copied()
            .collect()
    }

    // A refactor !
    pub fn scan_for_free_ssi(
        &self,
        len: Option<u16>,
        ref_si: Option<u16>,
        slots_count: Option<u8>,
        chn: Channel,
    ) -> Box<[u16]> {
        let len: u16 = len.unwrap_or(1);
        let ref_si: u16 = ref_si.unwrap_or(SlotsMap::current_si(chn.clone()));
        let end_si: u16 = SlotsMap::offseted_si(ref_si, len);
        let ssi_count: u8 = slots_count.unwrap_or(1);

        match chn {
            Channel::C87B | Channel::C88B => {
                let ssi_range: Box<[u16]> = self.ssi_range(ref_si, end_si, chn);
                let available_ssi: Box<[u16]> = self.filter_unavailable_ssi(ssi_range);

                available_ssi
            }
            Channel::Any => {
                let c87b_ssi_range: Box<[u16]> = self.ssi_range(ref_si, end_si, Channel::C87B);
                let c88b_ssi_range: Box<[u16]> = self.ssi_range(ref_si, end_si, Channel::C88B);

                let available_c87b_ssi: Box<[u16]> = self.filter_unavailable_ssi(c87b_ssi_range);
                let available_c88b_ssi: Box<[u16]> = self.filter_unavailable_ssi(c88b_ssi_range);

                let is_c87b_ssi_range_valid: bool =
                    available_c87b_ssi.len() >= 4.max(ssi_count as usize);
                let is_c88b_ssi_range_valid: bool =
                    available_c88b_ssi.len() >= 4.max(ssi_count as usize);

                if is_c87b_ssi_range_valid && is_c88b_ssi_range_valid {
                    let chosen_chn: &Channel = [Channel::C87B, Channel::C88B]
                        .choose(&mut rand::rng())
                        .unwrap();

                    match chosen_chn {
                        Channel::C87B => Box::from(available_c87b_ssi),
                        Channel::C88B => Box::from(available_c88b_ssi),
                        _ => Box::from([]),
                    }
                } else if is_c87b_ssi_range_valid {
                    Box::from(available_c87b_ssi)
                } else if is_c88b_ssi_range_valid {
                    Box::from(available_c88b_ssi)
                } else {
                    Box::from([])
                }
            }
            _ => Box::from([]),
        }
    }

    pub fn scan_for_self_owned_ssi(
        &self,
        len: Option<u16>,
        ref_si: Option<u16>,
        chn: Channel,
    ) -> Box<[u16]> {
        let length: u16 = len.unwrap_or(SLOTS_PER_MINUTE - 1);
        let ref_si: u16 = ref_si.unwrap_or(0);
        let end_si: u16 = SlotsMap::offseted_si(ref_si, length);

        let ssi_range: Box<[u16]> = self.ssi_range(ref_si, end_si, chn);

        let available_ssi: Vec<u16> = ssi_range
            .iter()
            .filter(|idx: &&u16| self.slot_owner(**idx) == Some(self.boat_mmsi))
            .copied()
            .collect();

        available_ssi.into_boxed_slice()
    }
}
