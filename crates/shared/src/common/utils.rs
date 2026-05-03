use crate::common::constants::{SIX_BITS_ASCII_ALPHABET, SLOTS_PER_MINUTE};
use chrono::{DateTime, Local, Timelike};

#[inline]
pub fn char6(ord: u8) -> char {
    SIX_BITS_ASCII_ALPHABET[(ord - 1) as usize] as char
}

#[inline]
pub fn ord6(chr: char) -> u8 {
    match chr as u8 {
        b'@' => 1,
        b'A' => 2,
        b'B' => 3,
        b'C' => 4,
        b'D' => 5,
        b'E' => 6,
        b'F' => 7,
        b'G' => 8,
        b'H' => 9,
        b'I' => 10,
        b'J' => 11,
        b'K' => 12,
        b'L' => 13,
        b'M' => 14,
        b'N' => 15,
        b'O' => 16,
        b'P' => 17,
        b'Q' => 18,
        b'R' => 19,
        b'S' => 20,
        b'T' => 21,
        b'U' => 22,
        b'V' => 23,
        b'W' => 24,
        b'X' => 25,
        b'Y' => 26,
        b'Z' => 27,
        b'[' => 28,
        b'\\' => 29,
        b']' => 30,
        b'^' => 31,
        b'_' => 32,
        b' ' => 33,
        b'!' => 34,
        b'"' => 35,
        b'#' => 36,
        b'$' => 37,
        b'%' => 38,
        b'&' => 39,
        b'\'' => 40,
        b'(' => 41,
        b')' => 42,
        b'*' => 43,
        b'+' => 44,
        b',' => 45,
        b'-' => 46,
        b'.' => 47,
        b'/' => 48,
        b'0' => 49,
        b'1' => 50,
        b'2' => 51,
        b'3' => 52,
        b'4' => 53,
        b'5' => 54,
        b'6' => 55,
        b'7' => 56,
        b'8' => 57,
        b'9' => 58,
        b':' => 59,
        b';' => 60,
        b'<' => 61,
        b'=' => 62,
        b'>' => 63,
        b'?' => 64,
        _ => 1,
    }
}

pub fn get_current_dt() -> DateTime<Local> {
    Local::now()
}

pub fn get_timestamp(datetime: Option<DateTime<Local>>) -> i64 {
    let datetime = datetime.unwrap_or(Local::now());
    datetime.timestamp()
}

#[inline]
pub fn dt_to_slots_idx(datetime: Option<DateTime<Local>>) -> [u16; 2] {
    let dt = datetime.unwrap_or(Local::now());

    let ns_since_min_start = (dt.second() as u64 * 1_000_000_000) + dt.nanosecond() as u64;

    let si = ((ns_since_min_start * 3) / 80_000_000) as u16;

    let si = si.min(SLOTS_PER_MINUTE - 1);

    [si, si + SLOTS_PER_MINUTE]
}
