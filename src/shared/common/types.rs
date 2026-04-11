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
pub type MessageResult<T> = Result<T, MessageError>;


impl From<BitPackerError> for MessageError {
    fn from(value: BitPackerError) -> Self {
        Self::BitPacker(value)
    }
}
