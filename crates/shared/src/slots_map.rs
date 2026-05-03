use crate::{
    common::{
        constants::SLOTS_PER_MINUTE,
        errors::{SlotsMapError, SlotsMapResult},
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

    pub async fn run_cleanup_task(&self) -> SlotsMapResult<()> {
        loop {
            for slot in self
                .slots
                .write()
                .map_err(|_| SlotsMapError::SlotsMapPoisoned)?
                .iter_mut()
            {
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

            let ns_since_epoch = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let ns_until_next_min = 60_000_000_000 - (ns_since_epoch % 60_000_000_000);

            let start_instant: Instant =
                Instant::now() + Duration::from_nanos(ns_until_next_min as u64);

            interval_at(start_instant, Duration::from_secs(60))
                .tick()
                .await;
        }
    }

    pub fn use_slot(&self, si: u16) -> SlotsMapResult<()> {
        self.slots
            .write()
            .map_err(|_| SlotsMapError::SlotsMapPoisoned)?[si as usize]
            .tick();
        Ok(())
    }

    pub fn flag_slot_as_used(&self, si: u16) -> SlotsMapResult<()> {
        self.slots
            .write()
            .map_err(|_| SlotsMapError::SlotsMapPoisoned)?[si as usize]
            .flag_as_used();
        Ok(())
    }

    pub fn slot_owner(&self, si: u16) -> SlotsMapResult<Option<u32>> {
        Ok(*self
            .slots
            .read()
            .map_err(|_| SlotsMapError::SlotsMapPoisoned)?[si as usize]
            .owner())
    }

    pub fn slot_timeout(&self, si: u16) -> SlotsMapResult<Option<u8>> {
        Ok(*self
            .slots
            .read()
            .map_err(|_| SlotsMapError::SlotsMapPoisoned)?[si as usize]
            .timeout())
    }

    pub fn slot_channel(&self, si: u16) -> SlotsMapResult<Channel> {
        Ok(*self
            .slots
            .read()
            .map_err(|_| SlotsMapError::SlotsMapPoisoned)?[si as usize]
            .channel())
    }

    pub fn is_slot_free(&self, si: u16) -> SlotsMapResult<bool> {
        Ok(self
            .slots
            .read()
            .map_err(|_| SlotsMapError::SlotsMapPoisoned)?[si as usize]
            .is_free())
    }

    pub fn is_slot_expired(&self, si: u16) -> SlotsMapResult<bool> {
        Ok(*self
            .slots
            .read()
            .map_err(|_| SlotsMapError::SlotsMapPoisoned)?[si as usize]
            .frames_since_last_use()
            > 2)
    }

    pub fn is_slot_current(&self, si: u16) -> SlotsMapResult<bool> {
        Ok(dt_to_slots_idx(None).contains(&si))
    }

    pub fn set_slot_timeout(&self, si: u16, timeout: Option<u8>) -> SlotsMapResult<()> {
        self.slots
            .write()
            .map_err(|_| SlotsMapError::SlotsMapPoisoned)?[si as usize]
            .set_timeout(timeout);

        Ok(())
    }

    pub fn book_slot(
        &self,
        si: u16,
        mmsi: u32,
        timeout: Option<u8>,
        is_assigned: Option<bool>,
    ) -> SlotsMapResult<()> {
        self.slots
            .write()
            .map_err(|_| SlotsMapError::SlotsMapPoisoned)?[si as usize]
            .book(mmsi, timeout, is_assigned.unwrap_or(false));

        Ok(())
    }

    pub fn release_slot(&self, si: u16) -> SlotsMapResult<()> {
        self.slots
            .write()
            .map_err(|_| SlotsMapError::SlotsMapPoisoned)?[si as usize]
            .release();

        Ok(())
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
        let s0 = s0.unwrap_or(SlotsMap::current_si(Channel::C87B));

        (s1 % SLOTS_PER_MINUTE + SLOTS_PER_MINUTE - s0 % SLOTS_PER_MINUTE) % SLOTS_PER_MINUTE
    }

    pub fn absolute_si_distance(s0: Option<u16>, s1: u16) -> u16 {
        let s0 = s0.unwrap_or(SlotsMap::current_si(Channel::C87B));

        (s0 % SLOTS_PER_MINUTE).abs_diff(s1 % SLOTS_PER_MINUTE)
    }

    pub fn offseted_si(si: u16, offset: u16) -> u16 {
        let offseted_si = (si + offset) % SLOTS_PER_MINUTE;

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

    pub fn get_available_ssi(&self, chn: Option<Channel>) -> SlotsMapResult<Box<[u16]>> {
        let target_chn = chn.unwrap_or(Channel::Any);

        (0..(2 * SLOTS_PER_MINUTE))
            .map(|si| {
                if self.is_slot_free(si)? {
                    let matches = match target_chn {
                        Channel::Any => true,
                        _ => self.slot_channel(si)? == target_chn,
                    };
                    if matches {
                        return Ok(Some(si));
                    }
                }
                Ok(None)
            })
            .filter_map(|res| res.transpose())
            .collect::<SlotsMapResult<Vec<u16>>>()
            .map(|v| v.into_boxed_slice())
    }

    pub fn filter_available_ssi(&self, slots: &[u16]) -> SlotsMapResult<Box<[u16]>> {
        slots
            .iter()
            .map(|&s| {
                self.is_slot_free(s)
                    .map(|free| if free { Some(s) } else { None })
            })
            .filter_map(|res| res.transpose()) // Transforme Result<Option<T>> en Option<Result<T>>
            .collect::<SlotsMapResult<Vec<_>>>()
            .map(|v| v.into_boxed_slice())
    }

    // A refactor !
    pub fn scan_for_free_ssi(
        &self,
        len: Option<u16>,
        ref_si: Option<u16>,
        slots_count: Option<u8>,
        chn: Channel,
    ) -> SlotsMapResult<Box<[u16]>> {
        let len = len.unwrap_or(1);
        let ref_si = ref_si.unwrap_or(SlotsMap::current_si(chn.clone()));
        let end_si = SlotsMap::offseted_si(ref_si, len);
        let ssi_count = slots_count.unwrap_or(1);

        match chn {
            Channel::C87B | Channel::C88B => {
                let ssi_range = self.ssi_range(ref_si, end_si, chn);
                let available_ssi = self.filter_available_ssi(&ssi_range)?;

                Ok(available_ssi)
            }
            Channel::Any => {
                let c87b_ssi_range = self.ssi_range(ref_si, end_si, Channel::C87B);
                let c88b_ssi_range = self.ssi_range(ref_si, end_si, Channel::C88B);

                let available_c87b_ssi = self.filter_available_ssi(&c87b_ssi_range)?;
                let available_c88b_ssi = self.filter_available_ssi(&c88b_ssi_range)?;

                let is_c87b_ssi_range_valid: bool =
                    available_c87b_ssi.len() >= 4.max(ssi_count as usize);
                let is_c88b_ssi_range_valid: bool =
                    available_c88b_ssi.len() >= 4.max(ssi_count as usize);

                if is_c87b_ssi_range_valid && is_c88b_ssi_range_valid {
                    let chosen_chn = [Channel::C87B, Channel::C88B]
                        .choose(&mut rand::rng())
                        .unwrap();

                    match chosen_chn {
                        Channel::C87B => Ok(Box::from(available_c87b_ssi)),
                        Channel::C88B => Ok(Box::from(available_c88b_ssi)),
                        _ => Ok(Box::from([])),
                    }
                } else if is_c87b_ssi_range_valid {
                    Ok(Box::from(available_c87b_ssi))
                } else if is_c88b_ssi_range_valid {
                    Ok(Box::from(available_c88b_ssi))
                } else {
                    Ok(Box::from([]))
                }
            }
            _ => Ok(Box::from([])),
        }
    }

    pub fn scan_for_self_owned_ssi(
        &self,
        len: Option<u16>,
        ref_si: Option<u16>,
        chn: Channel,
    ) -> SlotsMapResult<Box<[u16]>> {
        let length = len.unwrap_or(SLOTS_PER_MINUTE - 1);
        let start_si = ref_si.unwrap_or(0);
        let end_si = SlotsMap::offseted_si(start_si, length);

        let ssi_range = self.ssi_range(start_si, end_si, chn);

        ssi_range
            .iter()
            .map(|&si| {
                // On récupère le propriétaire. Si la lecture échoue (ex: verrou empoisonné), on propage l'erreur.
                self.slot_owner(si).map(|owner| {
                    if owner == Some(self.boat_mmsi) {
                        Some(si)
                    } else {
                        None
                    }
                })
            })
            .filter_map(|res| res.transpose()) // Transforme Result<Option<u16>> en Option<Result<u16>>
            .collect::<SlotsMapResult<Vec<_>>>()
            .map(|v| v.into_boxed_slice())
    }
}
