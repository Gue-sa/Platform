use std::fmt::Binary;

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


pub fn pad_left(bits: &str, target_size: usize) -> String {
    format!("{:0>target_size$}", bits)
}


pub fn uint_to_bits<T: Binary>(nbr: T, bits_size: Option<usize>) -> String {
    let bits_size: usize = bits_size.unwrap_or(0);

    if bits_size == 0 {
        format!("{:b}", nbr)
    } else {
        format!("{:0>bits_size$b}", nbr)
    }
}


pub fn u8_to_bits(nbr: u8, bits_size: Option<usize>) -> String {
    let bits_size: usize = bits_size.unwrap_or(0);

    if bits_size == 0 {
        format!("{:b}", nbr)
    } else {
        format!("{:0>bits_size$b}", nbr)
    }
}


pub fn bits_to_string(bits: &str) -> String {
    let mut processed_bits: String = String::from(bits);
    let mut converted_string: String = String::from("");

    let processed_bits_length: usize = processed_bits.len();

    if processed_bits_length % 6 != 0 {
        processed_bits = pad_left(&processed_bits, processed_bits_length + 6 - (processed_bits_length % 6));
    }

    while &processed_bits[0..6] == "000000" {
        processed_bits.drain(0..6);
    }

    while processed_bits.len() != 0 {
        let bits_slice: &str = &processed_bits[0..6];
        converted_string = format!("{}{}", char6(u8::from_str_radix(bits_slice, 2).unwrap()), converted_string);
        processed_bits.drain(0..6);
    }

    converted_string
}


pub fn string_to_bits(string: &str, bits_size: Option<usize>) -> String {
    let mut processed_string = String::from(string);
    let mut converted_bits = String::from("");
    let bits_size: usize = bits_size.unwrap_or(0);
    
    while processed_string.len() != 0 {
        let first_char: Option<char> = processed_string.chars().next();
        match first_char {
            Some(c) => {
                converted_bits = format!("{}{}", u8_to_bits(ord6(c), Some(6)), converted_bits);
                processed_string.drain(0..1);
            },
            None => ()
        }
    }

    if bits_size == 0 {
        converted_bits
    } else {
        pad_left(&converted_bits, bits_size)
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
    println!("[{}, {}] : {}\n", slots[0], slots[1], msg);
}