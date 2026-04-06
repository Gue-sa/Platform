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

pub const MSG123_FIELDS: [&str; 13] = [
    "mmsi",
    "navigational_status",
    "rate_of_turn",
    "speed_over_ground",
    "position_accuracy",
    "longitude",
    "latitude",
    "course_over_ground",
    "true_heading",
    "time_stamp",
    "special_maneuvre_indicator",
    "spare",
    "raim_flag"
];

pub const MSG5_FIELDS: [&str; 19] = [
    "mmsi",
    "ais_version",
    "imo_number",
    "call_sign",
    "name",
    "type_of_ship_and_cargo_type",
    "a",
    "b",
    "c",
    "d",
    "type_of_epf_device",
    "eta_minute",
    "eta_hour",
    "eta_day",
    "eta_month",
    "maximum_present_static_draught",
    "destination",
    "dte",
    "spare"
];

pub const BOAT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(10,0,0,1));