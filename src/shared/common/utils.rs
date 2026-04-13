use crate::shared::common::constants::SIX_BITS_ASCII_ALPHABET;

pub fn char6(ord: u8) -> char {
    SIX_BITS_ASCII_ALPHABET[usize::from(ord - 1)] as char
}

pub fn ord6(char: char) -> u8 {
    let index = SIX_BITS_ASCII_ALPHABET
        .iter()
        .position(|&c| c == char as u8);

    match index {
        Some(ord) => ord as u8 + 1,
        None => 1,
    }
}
