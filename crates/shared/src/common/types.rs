use crate::bitpacker::BitPacker;

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
    Unattributed,
    UnderRevision,
    RevisionSubmitted,
    Accepted,
    Refused,
    InExecution,
    Completed,
    Finished,
    Aborted,
    Unknown,
}

impl Into<VoyageStatus> for u8 {
    fn into(self) -> VoyageStatus {
        match self {
            0 => VoyageStatus::Unattributed,
            1 => VoyageStatus::UnderRevision,
            2 => VoyageStatus::RevisionSubmitted,
            3 => VoyageStatus::Accepted,
            4 => VoyageStatus::Refused,
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
            VoyageStatus::Unattributed => 0,
            VoyageStatus::UnderRevision => 1,
            VoyageStatus::RevisionSubmitted => 2,
            VoyageStatus::Accepted => 3,
            VoyageStatus::Refused => 4,
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
    Acceptation,
    Refusal,
    Revision,
    Executing,
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
            3 => SatComMessageType::Acceptation,
            4 => SatComMessageType::Refusal,
            5 => SatComMessageType::Revision,
            6 => SatComMessageType::Executing,
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
            SatComMessageType::Acceptation => 3,
            SatComMessageType::Refusal => 4,
            SatComMessageType::Revision => 5,
            SatComMessageType::Executing => 6,
            SatComMessageType::Aborting => 7,
            SatComMessageType::NoticeOfReadiness => 8,
            SatComMessageType::EndOfVoyage => 9,
            SatComMessageType::Unknown => u8::MAX,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ClockError {
    SlotOvershoot
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum BitPackerError {
    IndexOutOfBounds,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AisMessageError {
    UnknownMessageType,
    UnkownSotdmaTimeout,
    CrcMismatch,
    BitPacker(BitPackerError),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AisError {
    SelfEmittedMessage,
    NoFreeSlot,
    NoOwnedSlot,
    NoValidSlotSelection,
    SotdmaInitFailed,
    AisMessage(AisMessageError),
    Clock(ClockError)
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum VoyageOrderError {
    MalformedVoyageOrder,
    BitPacker(BitPackerError),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum SatComMessageError {
    BitPacker(BitPackerError),
    UnknownSatComMessageType,
    VoyageOrder(VoyageOrderError),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum SatComError {
    UnknownMessageType,
    SatComMessage(SatComMessageError),
}

#[derive(Debug, PartialEq)]
pub enum DatabaseManagerError {
    InsertionError(diesel::result::Error),
    QueryError(diesel::result::Error)
}

pub type ClockResult<T> = Result<T, ClockError>;
pub type BitPackerResult<T> = Result<T, BitPackerError>;
pub type AisResult<T> = Result<T, AisError>;
pub type AisMessageResult<T> = Result<T, AisMessageError>;
pub type VoyageOrderResult<T> = Result<T, VoyageOrderError>;
pub type SatComResult<T> = Result<T, SatComError>;
pub type SatComMessageResult<T> = Result<T, SatComMessageError>;
pub type DatabaseManagerResult<T> = Result<T, DatabaseManagerError>;

impl From<BitPackerError> for AisMessageError {
    fn from(value: BitPackerError) -> Self {
        Self::BitPacker(value)
    }
}

impl From<BitPackerError> for SatComMessageError {
    fn from(value: BitPackerError) -> Self {
        Self::BitPacker(value)
    }
}

impl From<BitPackerError> for VoyageOrderError {
    fn from(value: BitPackerError) -> Self {
        Self::BitPacker(value)
    }
}

impl From<ClockError> for AisError {
    fn from(value: ClockError) -> Self {
        Self::Clock(value)
    }
}

impl From<AisMessageError> for AisError {
    fn from(value: AisMessageError) -> Self {
        Self::AisMessage(value)
    }
}

impl From<VoyageOrderError> for SatComMessageError {
    fn from(value: VoyageOrderError) -> Self {
        Self::VoyageOrder(value)
    }
}

impl From<SatComMessageError> for SatComError {
    fn from(value: SatComMessageError) -> Self {
        Self::SatComMessage(value)
    }
}
