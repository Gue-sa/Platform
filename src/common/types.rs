use std::net::IpAddr;

use crate::shared::bitpacker::BitPacker;

#[derive(Copy, Clone)]
pub enum Channel {
    C87B,
    C88B,
    GPS,
    Any
}


pub struct Packet {
    pub message: BitPacker,
    pub channel: Channel
}