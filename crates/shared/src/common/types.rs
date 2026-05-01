use std::{io::Error, time::SystemTimeError};

use colored::ColoredString;
use tokio::sync::mpsc::error::SendError;

use crate::{bitpacker::BitPacker, satcom_message::SatComMessage};

#[derive(Debug, Clone, PartialEq)]
pub struct AisPacket {
    pub channel: Channel,
    pub message: BitPacker,
}

impl AisPacket {
    pub fn from(msg: BitPacker, chn: Channel) -> Self {
        Self {
            channel: chn,
            message: msg,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Channel {
    C87B,
    C88B,
    GPS,
    SATCOM,
    Any,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CSType {
    SOTDMA,
    ITDMA,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum VoyageStatus {
    Unassigned,
    UnderRevision,
    RevisionSubmitted,
    RevisionAccepted,
    RevisionRefused,
    InExecution,
    Completed,
    Finished,
    Aborted,
    Unknown,
}

impl Into<VoyageStatus> for u8 {
    fn into(self) -> VoyageStatus {
        match self {
            0 => VoyageStatus::Unassigned,
            1 => VoyageStatus::UnderRevision,
            2 => VoyageStatus::RevisionSubmitted,
            3 => VoyageStatus::RevisionAccepted,
            4 => VoyageStatus::RevisionRefused,
            5 => VoyageStatus::InExecution,
            6 => VoyageStatus::Completed,
            7 => VoyageStatus::Finished,
            8 => VoyageStatus::Aborted,
            _ => VoyageStatus::Unknown,
        }
    }
}

impl Into<u8> for VoyageStatus {
    fn into(self) -> u8 {
        match self {
            VoyageStatus::Unassigned => 0,
            VoyageStatus::UnderRevision => 1,
            VoyageStatus::RevisionSubmitted => 2,
            VoyageStatus::RevisionAccepted => 3,
            VoyageStatus::RevisionRefused => 4,
            VoyageStatus::InExecution => 5,
            VoyageStatus::Completed => 6,
            VoyageStatus::Finished => 7,
            VoyageStatus::Aborted => 8,
            VoyageStatus::Unknown => 9,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum SatComMessageType {
    Offer,
    Acknowledgement,
    InitialRevision,
    RevisionAcceptation,
    RevisionRefusal,
    RevisionRequest,
    ExecutingLastAgreedRevision,
    Aborting,
    NoticeOfReadiness,
    EndOfVoyage,
    Unknown,
}

impl Into<SatComMessageType> for u8 {
    fn into(self) -> SatComMessageType {
        match self {
            0 => SatComMessageType::Offer,
            1 => SatComMessageType::Acknowledgement,
            2 => SatComMessageType::InitialRevision,
            3 => SatComMessageType::RevisionAcceptation,
            4 => SatComMessageType::RevisionRefusal,
            5 => SatComMessageType::RevisionRequest,
            6 => SatComMessageType::ExecutingLastAgreedRevision,
            7 => SatComMessageType::Aborting,
            8 => SatComMessageType::NoticeOfReadiness,
            9 => SatComMessageType::EndOfVoyage,
            _ => SatComMessageType::Unknown,
        }
    }
}

impl Into<u8> for SatComMessageType {
    fn into(self) -> u8 {
        match self {
            SatComMessageType::Offer => 0,
            SatComMessageType::Acknowledgement => 1,
            SatComMessageType::InitialRevision => 2,
            SatComMessageType::RevisionAcceptation => 3,
            SatComMessageType::RevisionRefusal => 4,
            SatComMessageType::RevisionRequest => 5,
            SatComMessageType::ExecutingLastAgreedRevision => 6,
            SatComMessageType::Aborting => 7,
            SatComMessageType::NoticeOfReadiness => 8,
            SatComMessageType::EndOfVoyage => 9,
            SatComMessageType::Unknown => u8::MAX,
        }
    }
}

pub enum ShipType {
    ScientificResearch,
    TrainingShip,
    GovernmentVessel,
    Icebreaker,
    BuoyTender,
    CableLayer,
    PipeLayer,
    SpecialPurposeNoInfo,
    Fpso,
    FactoryShip,
    FishFarmSupport,
    OffshoreSupport,
    ConstructionVessel,
    Crewboat,
    SupportVesselNoInfo,
    WigAll,
    WigHazardousCatX,
    WigHazardousCatY,
    WigHazardousCatZ,
    WigHazardousCatOs,
    WigNoInfo,
    FishingVessel,
    Tug,
    LargeTug,
    Dredger,
    DiveVessel,
    MilitaryNavalAuxiliary,
    SailingVessel,
    PleasureCraftMotor,
    Trawler,
    PatrolVessel,
    HscAll,
    HscHazardousCatX,
    HscHazardousCatY,
    HscHazardousCatZ,
    HscHazardousCatOs,
    HscPassenger,
    HscRoRo,
    HscNoInfo,
    PilotBoat,
    SarVessel,
    Tugs,
    PortFishingTender,
    AntiPollutionFireVessel,
    LawEnforcement,
    LocalVessel1,
    LocalVessel2,
    MedicalTransport,
    NonBelligerentState,
    PassengerAll,
    PassengerHazardousCatX,
    PassengerHazardousCatY,
    PassengerHazardousCatZ,
    PassengerHazardousCatOs,
    PassengerCruise,
    PassengerFerry,
    PassengerExcursion,
    PassengerNoInfo,
    CargoAll,
    CargoHazardousCatX,
    CargoHazardousCatY,
    CargoHazardousCatZ,
    CargoHazardousCatOs,
    CargoBulk,
    CargoContainer,
    CargoRoRo,
    CargoLandingCraft,
    CargoNoInfo,
    TankerAll,
    TankerHazardousCatX,
    TankerHazardousCatY,
    TankerHazardousCatZ,
    TankerHazardousCatOs,
    TankerNonHazardous,
    TankerArticulatedTugBarge,
    TankerNoInfo,
    OtherAll,
    OtherHazardousCatX,
    OtherHazardousCatY,
    OtherHazardousCatZ,
    OtherHazardousCatOs,
    OtherNoInfo,
    Unknown,
}

impl Into<ShipType> for u8 {
    fn into(self) -> ShipType {
        match self {
            1 => ShipType::ScientificResearch,
            2 => ShipType::TrainingShip,
            3 => ShipType::GovernmentVessel,
            4 => ShipType::Icebreaker,
            5 => ShipType::BuoyTender,
            6 => ShipType::CableLayer,
            7 => ShipType::PipeLayer,
            9 => ShipType::SpecialPurposeNoInfo,
            11 => ShipType::Fpso,
            12 => ShipType::FactoryShip,
            13 => ShipType::FishFarmSupport,
            14 => ShipType::OffshoreSupport,
            17 => ShipType::ConstructionVessel,
            18 => ShipType::Crewboat,
            19 => ShipType::SupportVesselNoInfo,
            20 => ShipType::WigAll,
            21 => ShipType::WigHazardousCatX,
            22 => ShipType::WigHazardousCatY,
            23 => ShipType::WigHazardousCatZ,
            24 => ShipType::WigHazardousCatOs,
            29 => ShipType::WigNoInfo,
            30 => ShipType::FishingVessel,
            31 => ShipType::Tug,
            32 => ShipType::LargeTug,
            33 => ShipType::Dredger,
            34 => ShipType::DiveVessel,
            35 => ShipType::MilitaryNavalAuxiliary,
            36 => ShipType::SailingVessel,
            37 => ShipType::PleasureCraftMotor,
            38 => ShipType::Trawler,
            39 => ShipType::PatrolVessel,
            40 => ShipType::HscAll,
            41 => ShipType::HscHazardousCatX,
            42 => ShipType::HscHazardousCatY,
            43 => ShipType::HscHazardousCatZ,
            44 => ShipType::HscHazardousCatOs,
            45 => ShipType::HscPassenger,
            46 => ShipType::HscRoRo,
            49 => ShipType::HscNoInfo,
            50 => ShipType::PilotBoat,
            51 => ShipType::SarVessel,
            52 => ShipType::Tugs,
            53 => ShipType::PortFishingTender,
            54 => ShipType::AntiPollutionFireVessel,
            55 => ShipType::LawEnforcement,
            56 => ShipType::LocalVessel1,
            57 => ShipType::LocalVessel2,
            58 => ShipType::MedicalTransport,
            59 => ShipType::NonBelligerentState,
            60 => ShipType::PassengerAll,
            61 => ShipType::PassengerHazardousCatX,
            62 => ShipType::PassengerHazardousCatY,
            63 => ShipType::PassengerHazardousCatZ,
            64 => ShipType::PassengerHazardousCatOs,
            65 => ShipType::PassengerCruise,
            66 => ShipType::PassengerFerry,
            67 => ShipType::PassengerExcursion,
            69 => ShipType::PassengerNoInfo,
            70 => ShipType::CargoAll,
            71 => ShipType::CargoHazardousCatX,
            72 => ShipType::CargoHazardousCatY,
            73 => ShipType::CargoHazardousCatZ,
            74 => ShipType::CargoHazardousCatOs,
            75 => ShipType::CargoBulk,
            76 => ShipType::CargoContainer,
            77 => ShipType::CargoRoRo,
            78 => ShipType::CargoLandingCraft,
            79 => ShipType::CargoNoInfo,
            80 => ShipType::TankerAll,
            81 => ShipType::TankerHazardousCatX,
            82 => ShipType::TankerHazardousCatY,
            83 => ShipType::TankerHazardousCatZ,
            84 => ShipType::TankerHazardousCatOs,
            85 => ShipType::TankerNonHazardous,
            86 => ShipType::TankerArticulatedTugBarge,
            89 => ShipType::TankerNoInfo,
            90 => ShipType::OtherAll,
            91 => ShipType::OtherHazardousCatX,
            92 => ShipType::OtherHazardousCatY,
            93 => ShipType::OtherHazardousCatZ,
            94 => ShipType::OtherHazardousCatOs,
            99 => ShipType::OtherNoInfo,
            _ => ShipType::Unknown,
        }
    }
}

impl Into<u8> for ShipType {
    fn into(self) -> u8 {
        match self {
            ShipType::ScientificResearch => 1,
            ShipType::TrainingShip => 2,
            ShipType::GovernmentVessel => 3,
            ShipType::Icebreaker => 4,
            ShipType::BuoyTender => 5,
            ShipType::CableLayer => 6,
            ShipType::PipeLayer => 7,
            ShipType::SpecialPurposeNoInfo => 9,
            ShipType::Fpso => 11,
            ShipType::FactoryShip => 12,
            ShipType::FishFarmSupport => 13,
            ShipType::OffshoreSupport => 14,
            ShipType::ConstructionVessel => 17,
            ShipType::Crewboat => 18,
            ShipType::SupportVesselNoInfo => 19,
            ShipType::WigAll => 20,
            ShipType::WigHazardousCatX => 21,
            ShipType::WigHazardousCatY => 22,
            ShipType::WigHazardousCatZ => 23,
            ShipType::WigHazardousCatOs => 24,
            ShipType::WigNoInfo => 29,
            ShipType::FishingVessel => 30,
            ShipType::Tug => 31,
            ShipType::LargeTug => 32,
            ShipType::Dredger => 33,
            ShipType::DiveVessel => 34,
            ShipType::MilitaryNavalAuxiliary => 35,
            ShipType::SailingVessel => 36,
            ShipType::PleasureCraftMotor => 37,
            ShipType::Trawler => 38,
            ShipType::PatrolVessel => 39,
            ShipType::HscAll => 40,
            ShipType::HscHazardousCatX => 41,
            ShipType::HscHazardousCatY => 42,
            ShipType::HscHazardousCatZ => 43,
            ShipType::HscHazardousCatOs => 44,
            ShipType::HscPassenger => 45,
            ShipType::HscRoRo => 46,
            ShipType::HscNoInfo => 49,
            ShipType::PilotBoat => 50,
            ShipType::SarVessel => 51,
            ShipType::Tugs => 52,
            ShipType::PortFishingTender => 53,
            ShipType::AntiPollutionFireVessel => 54,
            ShipType::LawEnforcement => 55,
            ShipType::LocalVessel1 => 56,
            ShipType::LocalVessel2 => 57,
            ShipType::MedicalTransport => 58,
            ShipType::NonBelligerentState => 59,
            ShipType::PassengerAll => 60,
            ShipType::PassengerHazardousCatX => 61,
            ShipType::PassengerHazardousCatY => 62,
            ShipType::PassengerHazardousCatZ => 63,
            ShipType::PassengerHazardousCatOs => 64,
            ShipType::PassengerCruise => 65,
            ShipType::PassengerFerry => 66,
            ShipType::PassengerExcursion => 67,
            ShipType::PassengerNoInfo => 69,
            ShipType::CargoAll => 70,
            ShipType::CargoHazardousCatX => 71,
            ShipType::CargoHazardousCatY => 72,
            ShipType::CargoHazardousCatZ => 73,
            ShipType::CargoHazardousCatOs => 74,
            ShipType::CargoBulk => 75,
            ShipType::CargoContainer => 76,
            ShipType::CargoRoRo => 77,
            ShipType::CargoLandingCraft => 78,
            ShipType::CargoNoInfo => 79,
            ShipType::TankerAll => 80,
            ShipType::TankerHazardousCatX => 81,
            ShipType::TankerHazardousCatY => 82,
            ShipType::TankerHazardousCatZ => 83,
            ShipType::TankerHazardousCatOs => 84,
            ShipType::TankerNonHazardous => 85,
            ShipType::TankerArticulatedTugBarge => 86,
            ShipType::TankerNoInfo => 89,
            ShipType::OtherAll => 90,
            ShipType::OtherHazardousCatX => 91,
            ShipType::OtherHazardousCatY => 92,
            ShipType::OtherHazardousCatZ => 93,
            ShipType::OtherHazardousCatOs => 94,
            ShipType::OtherNoInfo => 99,
            ShipType::Unknown => u8::MAX,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum UnitType {
    Server,
    Harbourmaster,
    Boat,
    Unkown,
}

impl Into<UnitType> for u8 {
    fn into(self) -> UnitType {
        match self {
            0 => UnitType::Server,
            1 => UnitType::Harbourmaster,
            2 => UnitType::Boat,
            _ => UnitType::Unkown,
        }
    }
}

impl Into<u8> for UnitType {
    fn into(self) -> u8 {
        match self {
            UnitType::Server => 0,
            UnitType::Harbourmaster => 1,
            UnitType::Boat => 2,
            UnitType::Unkown => u8::MAX,
        }
    }
}

pub enum SpeedProfile {
    Economic,
    Classic,
    Fast,
    Unknown,
}

impl Into<SpeedProfile> for u8 {
    fn into(self) -> SpeedProfile {
        match self {
            0 => SpeedProfile::Economic,
            1 => SpeedProfile::Classic,
            2 => SpeedProfile::Fast,
            _ => SpeedProfile::Unknown,
        }
    }
}

impl Into<u8> for SpeedProfile {
    fn into(self) -> u8 {
        match self {
            SpeedProfile::Economic => 0,
            SpeedProfile::Classic => 1,
            SpeedProfile::Fast => 2,
            SpeedProfile::Unknown => u8::MAX,
        }
    }
}

pub enum LogEvent {
    System(ColoredString),
    Ais(ColoredString),
    Gps(ColoredString),
    Satcom(ColoredString),
    Computer(ColoredString),
}
