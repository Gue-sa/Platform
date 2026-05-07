use crate::common::{constants::SLOTS_PER_MINUTE, types::Channel};
use getset::{Getters, Setters};

#[derive(Debug, Getters, Setters)]
pub struct Slot {
    #[getset(get = "pub")]
    number: u16,
    #[getset(get = "pub")]
    channel: Channel,
    #[getset(get = "pub", set = "pub")]
    is_assigned: bool,
    #[getset(get = "pub", set = "pub")]
    owner: Option<u32>,
    #[getset(get = "pub", set = "pub")]
    timeout: Option<u8>,
    #[getset(get = "pub", set = "pub")]
    frames_since_last_use: i8,
}

impl Slot {
    pub fn new(nbr: u16) -> Self {
        Self {
            number: nbr,
            channel: Slot::idx_to_channel(nbr).unwrap(),
            is_assigned: false,
            owner: None,
            timeout: None,
            frames_since_last_use: -2,
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

    pub fn flag_as_used(&mut self) {
        self.frames_since_last_use = -1;
    }

    pub fn book(&mut self, mmsi: u32, timeout: Option<u8>, is_assigned: bool) {
        if self.owner.is_none() {
            self.owner = Some(mmsi);
            self.timeout = timeout;
            self.is_assigned = is_assigned;
            self.frames_since_last_use = 0;
        }
    }

    pub fn release(&mut self) {
        self.owner = None;
        self.timeout = None;
        self.is_assigned = false;
        self.frames_since_last_use = -2;
    }

    pub fn tick(&mut self) {
        self.flag_as_used();

        if self.timeout.unwrap() == 0 {
            self.release();
        } else if 0 < self.timeout.unwrap() {
            self.timeout = Some(self.timeout.unwrap() - 1);
        }
    }

    pub fn is_free(&self) -> bool {
        self.owner.is_none()
    }
}
