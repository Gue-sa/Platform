use std::{sync::{Arc, Mutex, MutexGuard}, thread};

use chrono::Timelike;
use rand::seq::IndexedRandom;

use crate::{common::{types::*, constants::*, utils::*}, slot::Slot};


pub struct SlotsMap {
    slots: Arc<Vec<Mutex<Slot>>>,
    mmsi: u32
}


impl SlotsMap {
    pub fn init(mmsi: u32) -> Self {
        let slots_map: Self = Self {
            slots: Arc::new(
            (0..2*SLOTS_PER_MINUTE)
                .map(|i| {
                    Mutex::new(Slot::init(i))
                }).collect::<Vec<Mutex<Slot>>>()
            ),
            mmsi: mmsi
        };

        let slots_cleanup_clone: Arc<Vec<Mutex<Slot>>> = Arc::clone(&slots_map.slots);

        thread::spawn(move || {
            let mut last_update_minute: u32 = get_current_datetime().minute();

            loop {
                if get_current_datetime().minute() != last_update_minute {
                    last_update_minute = get_current_datetime().minute();

                    for slot_mutex in slots_cleanup_clone.iter() {
                        let mut slot: MutexGuard<'_, Slot> = slot_mutex.lock().unwrap();
                        match slot.frames_since_last_use {
                            -2 => {
                                if !slot.is_free() {
                                    slot.release();
                                }
                            },
                            3 => slot.release(),
                            _ => slot.frames_since_last_use += 1
                        }
                    }
                }
            }
        });

        slots_map
    }


    pub fn slot_mutex(&self, si: u16) -> &Mutex<Slot> {
        &self.slots[si as usize]
    }


    pub fn use_slot(&self, si: u16) -> () {
        self.slot_mutex(si).lock().unwrap().tick();
    }


    pub fn mark_slot_as_used(&self, si: u16) -> () {
        self.slot_mutex(si).lock().unwrap().mark_as_used();
    }


    pub fn slot_owner(&self, si: u16) -> u32 {
        self.slot_mutex(si).lock().unwrap().owner
    }

    pub fn slot_timeout(&self, si: u16) -> i16 {
        self.slot_mutex(si).lock().unwrap().timeout
    }


    pub fn slot_channel(&self, si: u16) -> Channel {
        self.slot_mutex(si).lock().unwrap().channel
    }


    pub fn is_slot_free(&self, si: u16) -> bool {
        self.slot_mutex(si).lock().unwrap().is_free()
    }


    pub fn is_slot_expired(&self, si: u16) -> bool {
        self.slot_mutex(si).lock().unwrap().frames_since_last_use > 2
    }


    pub fn is_slot_current(&self, si: u16) -> bool {
        datetime_to_slots_idx(None).contains(&si)
    }


    pub fn book_slot(&self, si: u16, mmsi: u32, timeout: Option<i16>, assigned: Option<bool>) -> () {
        self.slot_mutex(si).lock().unwrap().book(mmsi, timeout.unwrap_or(-1), assigned.unwrap_or(false));
    }


    pub fn release_slot(&self, si: u16) -> () {
        self.slot_mutex(si).lock().unwrap().release();
    }


    pub fn current_slot_number(channel: Channel) -> u16 {
        let current_datetime: chrono::DateTime<chrono::Local> = get_current_datetime();
        match channel {
            Channel::C87B => datetime_to_slots_idx(Some(current_datetime))[0],
            Channel::C88B => datetime_to_slots_idx(Some(current_datetime))[1],
            _ => datetime_to_slots_idx(Some(current_datetime))[0]
        }
    }


    pub fn slot_offset(s0: Option<u16>, s1: u16) -> u16{
        let s0: u16 = s0.unwrap_or(SlotsMap::current_slot_number(Channel::C87B));

        (s1 % SLOTS_PER_MINUTE + SLOTS_PER_MINUTE - s0 % SLOTS_PER_MINUTE) % SLOTS_PER_MINUTE
    }


    pub fn absolute_slot_distance(s0: Option<u16>, s1: u16) -> u16 {
        let s0: u16 = s0.unwrap_or(SlotsMap::current_slot_number(Channel::C87B));

        (s0 % SLOTS_PER_MINUTE).abs_diff(s1 % SLOTS_PER_MINUTE)
    }


    pub fn offseted_slot(si: u16, offset: u16) -> u16 {
        let offseted_si: u16 = (si + offset) % SLOTS_PER_MINUTE;

        if si < SLOTS_PER_MINUTE {
            offseted_si
        } else {
            offseted_si + SLOTS_PER_MINUTE
        }
    }


    pub fn slots_idx_range(&self, start_si: u16, end_si: u16, channel: Channel) -> Box<[u16]> { // Prend en argument les slots % SLOTS_PER_MINUTE ! L'ajustement se fait tout seul en fonction de channel !
        if start_si <= end_si {
            match channel {
                Channel::C87B => (start_si..=end_si).collect(),
                Channel::C88B => (start_si + SLOTS_PER_MINUTE..= end_si + SLOTS_PER_MINUTE).collect(),
                Channel::Any => [self.slots_idx_range(start_si, end_si, Channel::C87B), self.slots_idx_range(start_si, end_si, Channel::C88B)].concat().into_boxed_slice(),
                _ => {Box::new([])}
            }
        } else {
            match channel {
                Channel::C87B => (start_si..SLOTS_PER_MINUTE).chain(0..=end_si).collect(),
                Channel::C88B => (start_si..2*SLOTS_PER_MINUTE).chain(SLOTS_PER_MINUTE..=end_si).collect(),
                Channel::Any => [self.slots_idx_range(start_si, end_si, Channel::C87B), self.slots_idx_range(start_si, end_si, Channel::C88B)].concat().into_boxed_slice(),
                _ => {Box::new([])}
            }
        }
    }


    pub fn available_slots_idx(&self, channel: Option<Channel>) -> Box<[u16]> {
        let channel: Channel = channel.unwrap_or(Channel::Any);

        match channel {
            Channel::Any => (0..self.slots.len() as u16).filter(|si: &u16| self.is_slot_free(*si)).collect(),
            Channel::C87B => (0..self.slots.len() as u16).filter(|si: &u16| self.is_slot_free(*si) && matches!(self.slot_channel(*si), Channel::C87B)).collect(),
            Channel::C88B => (0..self.slots.len() as u16).filter(|si: &u16| self.is_slot_free(*si) && matches!(self.slot_channel(*si), Channel::C88B)).collect(),
            _ => {Box::new([])}
        }
    }


    pub fn extract_available_slots_idx(&self, slots: Box<[u16]>) -> Box<[u16]> {
        slots.iter().filter(|slot_number: &&u16| self.is_slot_free(**slot_number)).copied().collect()
    }


    // A refactor !
    pub fn scan_for_free_slots(&self, length: Option<u16>, ref_si: Option<u16>, slots_count: Option<u8>, channel: Channel) -> Result<Box<[u16]>, &'static str> {
        let length: u16 = length.unwrap_or(1);
        let ref_si: u16 = ref_si.unwrap_or(SlotsMap::current_slot_number(channel.clone()));
        let end_si: u16 = SlotsMap::offseted_slot(ref_si, length);
        let slots_count: u8 = slots_count.unwrap_or(1);

        match channel {
            Channel::C87B | Channel::C88B => {
                let slots_range: Box<[u16]> = self.slots_idx_range(ref_si, end_si, channel);
                let available_slots: Box<[u16]> = self.extract_available_slots_idx(slots_range);
                let is_selection_feasible: bool = available_slots.len() >= 4.max(slots_count as usize);

                if is_selection_feasible {
                    Ok(Box::from(available_slots))
                } else {
                    Err("La sélection est impossible : nombre de slots disponible < 4 dans la configuration demandée.")
                }
            },
            Channel::Any => {
                let c_87_b_slots_range = self.slots_idx_range(ref_si, end_si, Channel::C87B);
                let c_88_b_slots_range = self.slots_idx_range(ref_si, end_si, Channel::C88B);
                let available_87_b_slots: Box<[u16]> = self.extract_available_slots_idx(c_87_b_slots_range);
                let available_88_b_slots: Box<[u16]> = self.extract_available_slots_idx(c_88_b_slots_range);
                let is_87_b_selection_feasible: bool = available_87_b_slots.len() >= 4.max(slots_count as usize);
                let is_88_b_selection_feasible: bool = available_88_b_slots.len() >= 4.max(slots_count as usize);

                if is_87_b_selection_feasible && is_88_b_selection_feasible {
                    let chosen_channel: &Channel = [Channel::C87B, Channel::C88B].choose(&mut rand::rng()).unwrap();

                    match chosen_channel {
                        Channel::C87B => {
                            Ok(Box::from(available_87_b_slots))
                        },
                        Channel::C88B => {
                            Ok(Box::from(available_88_b_slots))
                        },
                        _ => Err("La sélection est impossible : nombre de slots disponible < 4 dans la configuration demandée.")
                    }
                } else if is_87_b_selection_feasible {
                    Ok(Box::from(available_87_b_slots))
                } else if is_88_b_selection_feasible {
                    Ok(Box::from(available_88_b_slots))
                } else {
                    Err("La sélection est impossible : nombre de slots disponible < 4 dans la configuration demandée.")
                }
            },
            _ => Err("Channel invalide.")
        }
    }


    pub fn scan_for_self_owned_slots(&self, length: Option<u16>, ref_si: Option<u16>, channel: Channel) -> Result<Box<[u16]>, String> {
        let length: u16 = length.unwrap_or(SLOTS_PER_MINUTE - 1);
        let ref_si: u16 = ref_si.unwrap_or(0);
        let end_si: u16 = SlotsMap::offseted_slot(ref_si, length);

        let slots_range: Box<[u16]> = self.slots_idx_range(ref_si, end_si, channel);

        let available_slots: Vec<u16> = slots_range.iter().filter(|idx: &&u16| self.slot_owner(**idx) == self.mmsi).copied().collect();

        if available_slots.len() > 0 {
            Ok(available_slots.into_boxed_slice())
        } else {
            Err(format!("Aucun slot déjà réservé dans l'intervalle spécifiée. (start_si = {}, end_si = {})", ref_si, end_si))
        }
    }
}
