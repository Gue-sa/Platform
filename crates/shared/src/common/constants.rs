use std::net::{IpAddr, Ipv4Addr};

use crate::common::types::{AisField, AisMessageType};

pub const SIX_BITS_ASCII_ALPHABET: &[u8; 64] =
    b"@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_ !\"#$%&'()*+,-./0123456789:;<=>?";

pub const SLOTS_PER_MINUTE: u16 = 2250;
pub const SLOTS_DURATION: f64 = 60. / SLOTS_PER_MINUTE as f64;

pub const SOTDMA_CS_MSGS: [AisMessageType; 7] = [
    AisMessageType::Msg1,
    AisMessageType::Msg2,
    AisMessageType::Msg4,
    AisMessageType::Msg9,
    AisMessageType::Msg11,
    AisMessageType::Msg18,
    AisMessageType::Msg26,
];

pub const ITDMA_CS_MSGS: [AisMessageType; 4] = [
    AisMessageType::Msg3,
    AisMessageType::Msg9,
    AisMessageType::Msg18,
    AisMessageType::Msg26,
];

pub const NO_CS_MSGS: [AisMessageType; 19] = [
    AisMessageType::Msg5,
    AisMessageType::Msg6,
    AisMessageType::Msg7,
    AisMessageType::Msg8,
    AisMessageType::Msg10,
    AisMessageType::Msg12,
    AisMessageType::Msg13,
    AisMessageType::Msg14,
    AisMessageType::Msg15,
    AisMessageType::Msg16,
    AisMessageType::Msg17,
    AisMessageType::Msg19,
    AisMessageType::Msg20,
    AisMessageType::Msg21,
    AisMessageType::Msg22,
    AisMessageType::Msg23,
    AisMessageType::Msg24,
    AisMessageType::Msg25,
    AisMessageType::Msg27,
];

pub const IMPLEMENTED_MSGS: [AisMessageType; 4] = [
    AisMessageType::Msg1,
    AisMessageType::Msg2,
    AisMessageType::Msg3,
    AisMessageType::Msg5,
];

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

pub const FMS_UPDATE_SECS_INTERVAL: u64 = 30;

pub const HARBOURMASTER_IPADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
pub const SERVER_IPADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
