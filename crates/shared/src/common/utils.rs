use crate::common::constants::{SIX_BITS_ASCII_ALPHABET, SLOTS_PER_MINUTE};
use chrono::{DateTime, Local, Timelike};

pub fn char6(ord: u8) -> char {
    SIX_BITS_ASCII_ALPHABET[usize::from(ord - 1)] as char
}

pub fn ord6(chr: char) -> u8 {
    let idx = SIX_BITS_ASCII_ALPHABET.iter().position(|&c| c == chr as u8);

    match idx {
        Some(ord) => ord as u8 + 1,
        None => 1,
    }
}

pub fn get_current_dt() -> DateTime<Local> {
    Local::now()
}

pub fn get_timestamp(datetime: Option<DateTime<Local>>) -> i64 {
    let datetime: DateTime<Local> = datetime.unwrap_or(Local::now());
    datetime.timestamp()
}

pub fn dt_to_slots_idx(datetime: Option<DateTime<Local>>) -> [u16; 2] {
    let dt: DateTime<Local> = datetime.unwrap_or(Local::now());

    let ns_since_min_start: u64 = (dt.second() as u64 * 1_000_000_000) + dt.nanosecond() as u64;

    let si: u16 = ((ns_since_min_start * 3) / 80_000_000) as u16;

    let si: u16 = si.min(SLOTS_PER_MINUTE - 1);

    [si, si + SLOTS_PER_MINUTE]
}
