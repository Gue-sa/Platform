use crate::{bitpacker::BitPacker, common::types::VoyageOrderResult};

#[derive(Debug, Clone, PartialEq)]
pub struct VoyageOrderHeader {
    pub id: u32,
    pub version: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoyageOrderBody {
    pub destination: String,
    pub destination_position: (u32, u32),
    pub eta_month: u8,
    pub eta_day: u8,
    pub eta_hour: u8,
    pub eta_minute: u8,
    pub cargo_type: u8,
    pub speed_profile: u8, //0: eco, 1: à temps, 2: aussi vite que possible
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoyageOrder {
    pub header: VoyageOrderHeader,
    pub body: VoyageOrderBody,
}

impl VoyageOrderHeader {
    pub fn from(voyage_order_header_bitpacker: BitPacker) -> VoyageOrderResult<Self> {
        Ok(Self {
            id: voyage_order_header_bitpacker.extract_int::<u32>(None, Some(31))?,
            version: voyage_order_header_bitpacker.extract_int::<u8>(Some(32), None)?,
        })
    }

    pub fn to_bitpacker(&self) -> BitPacker {
        BitPacker::from_int(self.version, Some(8)) + BitPacker::from_int(self.id, Some(32))
    }
}

impl VoyageOrderBody {
    pub fn from(voyage_order_body_bitpacker: BitPacker) -> VoyageOrderResult<Self> {
        Ok(Self {
            destination: voyage_order_body_bitpacker.extract_str(None, Some(119))?,
            destination_position: (
                voyage_order_body_bitpacker.extract_int::<u32>(Some(120), Some(151))?,
                voyage_order_body_bitpacker.extract_int::<u32>(Some(152), Some(183))?,
            ),
            eta_month: voyage_order_body_bitpacker.extract_int::<u8>(Some(184), Some(191))?,
            eta_day: voyage_order_body_bitpacker.extract_int::<u8>(Some(192), Some(199))?,
            eta_hour: voyage_order_body_bitpacker.extract_int::<u8>(Some(200), Some(207))?,
            eta_minute: voyage_order_body_bitpacker.extract_int::<u8>(Some(208), Some(215))?,
            cargo_type: voyage_order_body_bitpacker.extract_int::<u8>(Some(216), Some(223))?,
            speed_profile: voyage_order_body_bitpacker.extract_int::<u8>(Some(224), None)?,
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
    }
}

impl VoyageOrder {
    pub fn from(voyage_order_bitpacker: BitPacker) -> VoyageOrderResult<Self> {
        Ok(Self {
            header: VoyageOrderHeader::from(voyage_order_bitpacker.slice(None, Some(39))?)?,
            body: VoyageOrderBody::from(voyage_order_bitpacker.slice(Some(40), None)?)?,
        })
    }

    pub fn to_bitpacker(&self) -> BitPacker {
        self.body.to_bitpacker() + self.header.to_bitpacker()
    }

    pub fn is_revision_of(&self, order2: &VoyageOrder) -> bool {
        self.header.id == order2.header.id && self.header.version > order2.header.version
    }
}
