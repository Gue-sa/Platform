use crate::shared::{bitpacker::BitPacker, common::types::VoyageOrderResult};

#[derive(Debug, Clone)]
pub struct VoyageOrder {
    pub id: u32,
    pub version: u8,
    pub destination: String,
    pub destination_position: (u32, u32),
    pub eta_month: u8,
    pub eta_day: u8,
    pub eta_hour: u8,
    pub eta_minute: u8,
    pub cargo_type: u8,
    pub speed_profile: u8, //0: eco, 1: à temps, 2: aussi vite que possible
}

impl VoyageOrder {
    pub fn from(voyage_order_bitpacker: BitPacker) -> VoyageOrderResult<Self> {
        Ok(Self {
            id: voyage_order_bitpacker.extract_int::<u32>(None, Some(31))?,
            version: voyage_order_bitpacker.extract_int::<u8>(Some(32), Some(39))?,
            destination: voyage_order_bitpacker.extract_str(Some(40), Some(159))?,
            destination_position: (
                voyage_order_bitpacker.extract_int::<u32>(Some(160), Some(191))?,
                voyage_order_bitpacker.extract_int::<u32>(Some(192), Some(223))?,
            ),
            eta_month: voyage_order_bitpacker.extract_int::<u8>(Some(224), Some(231))?,
            eta_day: voyage_order_bitpacker.extract_int::<u8>(Some(232), Some(239))?,
            eta_hour: voyage_order_bitpacker.extract_int::<u8>(Some(240), Some(247))?,
            eta_minute: voyage_order_bitpacker.extract_int::<u8>(Some(248), Some(255))?,
            cargo_type: voyage_order_bitpacker.extract_int::<u8>(Some(256), Some(263))?,
            speed_profile: voyage_order_bitpacker.extract_int::<u8>(Some(264), None)?,
        })
    }

    pub fn to_bitpacker(&self) -> BitPacker {
        BitPacker::from_int(self.speed_profile, Some(8))
            + BitPacker::from_int(self.eta_minute, Some(8))
            + BitPacker::from_int(self.eta_hour, Some(8))
            + BitPacker::from_int(self.eta_day, Some(8))
            + BitPacker::from_int(self.eta_month, Some(8))
            + BitPacker::from_int(self.destination_position.1, Some(32))
            + BitPacker::from_int(self.destination_position.0, Some(32))
            + BitPacker::from_str(&self.destination, Some(120))
            + BitPacker::from_int(self.version, Some(8))
            + BitPacker::from_int(self.id, Some(32))
    }
}
