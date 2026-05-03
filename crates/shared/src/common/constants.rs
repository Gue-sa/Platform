use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use crate::common::types::AisField;

pub const SIX_BITS_ASCII_ALPHABET: &[u8; 64] =
    b"@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_ !\"#$%&'()*+,-./0123456789:;<=>?";

pub const SLOTS_PER_MINUTE: u16 = 2250;
pub const SLOTS_DURATION: f64 = 60. / SLOTS_PER_MINUTE as f64;

pub const SOTDMA_CS_MSGS: [u8; 7] = [1, 2, 4, 9, 11, 18, 26];
pub const ITDMA_CS_MSGS: [u8; 4] = [3, 9, 18, 26];
pub const NO_CS_MSGS: [u8; 19] = [
    5, 6, 7, 8, 10, 12, 13, 14, 15, 16, 17, 19, 20, 21, 22, 23, 24, 25, 27,
];

pub const IMPLEMENTED_MSGS: [u8; 4] = [1, 2, 3, 5];

pub const C87B_TO_SERVER_PORT: u16 = 4444;
pub const C87B_FROM_SERVER_PORT: u16 = 5555;

pub const C88B_TO_SERVER_PORT: u16 = 6666;
pub const C88B_FROM_SERVER_PORT: u16 = 7777;

pub const GPS_TO_SERVER_PORT: u16 = 8888;
pub const GPS_FROM_SERVER_PORT: u16 = 9999;

pub const SATCOM_TO_SERVER_PORT: u16 = 8989;
pub const SATCOM_FROM_SERVER_PORT: u16 = 9898;

pub const HARBOURMASTER_MMSI: u32 = 0b111111111111111111111111111111;

pub const MSG123_FIELDS: [AisField; 13] = [
    AisField::Mmsi,
    AisField::NavigationalStatus,
    AisField::RateOfTurn,
    AisField::SpeedOverGround,
    AisField::PositionAccuracy,
    AisField::Longitude,
    AisField::Latitude,
    AisField::CourseOverGround,
    AisField::TrueHeading,
    AisField::TimeStamp,
    AisField::SpecialManeuvreIndicator,
    AisField::Spare,
    AisField::RaimFlag,
];

pub const MSG5_FIELDS: [AisField; 19] = [
    AisField::Mmsi,
    AisField::AisVersion,
    AisField::ImoNumber,
    AisField::CallSign,
    AisField::Name,
    AisField::TypeOfShipAndCargoType,
    AisField::A,
    AisField::B,
    AisField::C,
    AisField::D,
    AisField::TypeOfEpfDevice,
    AisField::EtaMinute,
    AisField::EtaHour,
    AisField::EtaDay,
    AisField::EtaMonth,
    AisField::MaximumPresentStaticDraught,
    AisField::Destination,
    AisField::Dte,
    AisField::Spare,
];

pub const FMS_UPDATE_SECS_INTERVAL: u64 = 1;

pub const HARBOURMASTER_IPADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
pub const SERVER_IPADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
