use crate::shared::common::types::{BitPackerError, MessageError};


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


pub type AisResult<T> = Result<T, AisError>;
pub type GpsResult<T> = Result<T, GpsError>;


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
