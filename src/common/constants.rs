use std::net::{IpAddr, Ipv4Addr};

pub const C87B_REC_PORT: u16 = 4444;
pub const C87B_EM_PORT: u16 = 5555;
pub const C88B_REC_PORT: u16 = 6666;
pub const C88B_EM_PORT: u16 = 7777;
pub const GPS_REC_PORT: u16 = 8888;
pub const GPS_EM_PORT: u16 = 9999;

pub const HARBOURMASTER_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(10,0,0,3));