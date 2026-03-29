use std::net::{IpAddr, Ipv4Addr};

pub const SIX_BITS_ASCII_ALPHABET: &[u8; 64] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";

pub const SLOTS_PER_MINUTE: u16 = 2250;
pub const SLOTS_DURATION: f64 = 60. / SLOTS_PER_MINUTE as f64;

pub const SOTDMA_CS_MSGS: [u8; 7] = [1, 2, 4, 9, 11, 18, 26];
pub const ITDMA_CS_MSGS: [u8; 4] = [3, 9, 18, 26];
pub const NO_CS_MSGS: [u8; 19] = [5, 6, 7, 8, 10, 12, 13, 14, 15, 16, 17, 19, 20, 21, 22, 23, 24, 25, 27];

pub const IMPLEMENTED_MSGS: [u8; 4] = [1, 2, 3, 5];

pub const C87B_REC_PORT: u16 = 4444;
pub const C87B_EM_PORT: u16 = 5555;
pub const C88B_REC_PORT: u16 = 6666;
pub const C88B_EM_PORT: u16 = 7777;
pub const GPS_REC_PORT: u16 = 8888;
pub const GPS_EM_PORT: u16 = 9999;

pub const BOAT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(10,0,0,1));