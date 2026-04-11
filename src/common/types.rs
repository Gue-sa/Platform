use crate::shared::bitpacker::BitPacker;

#[derive(Clone, Copy, Debug)]
pub enum Channel {
    C87B,
    C88B,
    GPS,
    Any
}


#[derive(Clone, Debug)]
pub enum CSTypes {
    SOTDMA,
    ITDMA
}


pub enum BitPackerError {
    IndexOutOfBounds
}


pub enum AisError {
    SelfEmittedMessage,
    NoFreeSlot,
    NoOwnedSlot,
    NoValidSlotSelection,
    SotdmaInitFailed,
    Message(MessageError)
}


pub enum GpsError {
    MalformedResponse,
    BitPacker(BitPackerError)
}


pub enum MessageError {
    UnknownMessageType,
    UnkownSotdmaTimeout,
    IncoherentArgumentsCombination,
    CrcMismatch,
    BitPacker(BitPackerError)
}


pub enum BoatInfoError {
    UnkownField
}


pub type BitPackerResult<T> = Result<T, BitPackerError>;
pub type AisResult<T> = Result<T, AisError>;
pub type GpsResult<T> = Result<T, GpsError>;
pub type MessageResult<T> = Result<T, MessageError>;


impl From<BitPackerError> for MessageError {
    fn from(value: BitPackerError) -> Self {
        Self::BitPacker(value)
    }
}


impl From<BitPackerError> for GpsError {
    fn from(value: BitPackerError) -> Self {
        Self::BitPacker(value)
    }
}


impl From<MessageError> for AisError {
    fn from(value: MessageError) -> Self {
        Self::Message(value)
    }
}
