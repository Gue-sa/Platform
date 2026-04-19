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
    QueryError(diesel::result::Error),
    UpdateError(diesel::result::Error),
    DeletionError(diesel::result::Error)
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
