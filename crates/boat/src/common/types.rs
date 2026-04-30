use shared::common::errors::BitPackerError;

pub enum GpsError {
    MalformedResponse,
    BitPacker(BitPackerError),
}

pub type GpsResult<T> = Result<T, GpsError>;

impl From<BitPackerError> for GpsError {
    fn from(value: BitPackerError) -> Self {
        Self::BitPacker(value)
    }
}
