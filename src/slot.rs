use crate::common::{types::*, constants::*};


pub struct Slot {
    pub number: u16,
    pub channel: Channel,
    pub assigned: bool,
    pub owner: u32,
    pub timeout: i16,
    pub frames_since_last_use: i16
}


impl Slot {
    pub fn init(number: u16) -> Self {
        Self {
            number: number,
            channel: Slot::idx_to_channel(number).unwrap(),
            assigned: false,
            owner: 0,
            timeout: -1,
            frames_since_last_use: -2
        }
    }


    pub fn idx_to_channel(slot_idx: u16) -> Result<Channel, &'static str> {
        if slot_idx < SLOTS_PER_MINUTE {
            Ok(Channel::C87B)
        } else if slot_idx >= SLOTS_PER_MINUTE && slot_idx <= 4500 {
            Ok(Channel::C88B)
        } else {
            Err("Numéro de slot invalide")
        }
    }


    pub fn mark_as_used(&mut self) -> () {
        self.frames_since_last_use = -1;
    }


    pub fn book(&mut self, mmsi: u32, timeout: i16, assigned: bool) -> () {
        if self.owner == 0 {
            self.owner = mmsi;
            self.timeout = timeout;
            self.assigned = assigned;
            self.frames_since_last_use = 0;
        }
    }


    pub fn release(&mut self) -> () {
        self.owner = 0;
        self.timeout = -1;
        self.assigned = false;
        self.frames_since_last_use = -2;
    }


    pub fn tick(&mut self) -> () {
        self.mark_as_used();
        
        if self.timeout == 0 {
            self.release();
        } else if -1 < self.timeout {
            self.timeout -= 1;
        }
    }


    pub fn is_free(&self) -> bool {
        self.owner == 0
    }
}