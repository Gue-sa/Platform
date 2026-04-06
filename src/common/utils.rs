use chrono::{DateTime, Local, Timelike};
use colored::ColoredString;

use crate::common::constants::*;


pub fn char6(ord: u8) -> char {
    SIX_BITS_ASCII_ALPHABET[usize::from(ord - 1)] as char
}


pub fn ord6(char: char) -> u8 {
    let index = SIX_BITS_ASCII_ALPHABET.iter().position(|&c| c == char as u8);

    match index {
        Some(ord) => ord as u8 + 1,
        None => 1
    }
}


pub fn get_current_datetime() -> DateTime<Local> {
    Local::now()
}


pub fn get_timestamp(datetime: Option<DateTime<Local>>) -> i64 {
    let datetime: DateTime<Local> = datetime.unwrap_or(Local::now());
    datetime.timestamp()
}


pub fn datetime_to_slots_idx(datetime: Option<DateTime<Local>>) -> [u16; 2] {
    let datetime: DateTime<Local> = datetime.unwrap_or(Local::now());
    let si: u16 = ((datetime.second() * 1000 + datetime.timestamp_subsec_millis()) * SLOTS_PER_MINUTE as u32 / 60_000) as u16;
    [si, si + SLOTS_PER_MINUTE]
}


pub fn log(msg: ColoredString) -> () {
    let slots: [u16; 2] = datetime_to_slots_idx(None);
    println!("[{}, {}] : {}", slots[0], slots[1], msg);
}