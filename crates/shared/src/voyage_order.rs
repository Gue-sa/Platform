use serde::Serialize;

use crate::{bitpacker::BitPacker, common::types::VoyageOrderResult};

use getset::{Getters, Setters};

#[derive(Debug, Clone, PartialEq, Serialize, Getters, Setters)]
pub struct VoyageOrderHeader {
    #[getset(get = "pub")]
    id: u16,
    #[getset(get = "pub", set = "pub")]
    version: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Getters, Setters)]
#[getset(get = "pub")]
pub struct VoyageOrderBody {
    destination: String,
    destination_position: (u16, u16),
    eta_month: u8,
    eta_day: u8,
    eta_hour: u8,
    eta_minute: u8,
    cargo_type: u8,
    speed_profile: u8, //0: eco, 1: à temps, 2: aussi vite que possible
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VoyageOrder {
    header: VoyageOrderHeader,
    body: VoyageOrderBody,
}

impl VoyageOrderHeader {
    pub fn from_bitpacker(voyage_order_header_bitpacker: BitPacker) -> VoyageOrderResult<Self> {
        Ok(Self {
            id: voyage_order_header_bitpacker.extract_int::<u16>(None, Some(15))?,
            version: voyage_order_header_bitpacker.extract_int::<u8>(Some(16), None)?,
        })
    }

    pub fn from_data(id: u16, version: u8) -> Self {
        Self {
            id: id,
            version: version,
        }
    }

    pub fn to_bitpacker(&self) -> BitPacker {
        BitPacker::from_int(self.version, Some(8)) + BitPacker::from_int(self.id, Some(32))
    }
}

impl VoyageOrderBody {
    pub fn from_bitpacker(voyage_order_body_bitpacker: BitPacker) -> VoyageOrderResult<Self> {
        Ok(Self {
            destination: voyage_order_body_bitpacker.extract_str(None, Some(119))?,
            destination_position: (
                voyage_order_body_bitpacker.extract_int::<u16>(Some(120), Some(135))?,
                voyage_order_body_bitpacker.extract_int::<u16>(Some(136), Some(151))?,
            ),
            eta_month: voyage_order_body_bitpacker.extract_int::<u8>(Some(152), Some(159))?,
            eta_day: voyage_order_body_bitpacker.extract_int::<u8>(Some(160), Some(167))?,
            eta_hour: voyage_order_body_bitpacker.extract_int::<u8>(Some(168), Some(175))?,
            eta_minute: voyage_order_body_bitpacker.extract_int::<u8>(Some(176), Some(183))?,
            cargo_type: voyage_order_body_bitpacker.extract_int::<u8>(Some(184), Some(191))?,
            speed_profile: voyage_order_body_bitpacker.extract_int::<u8>(Some(192), None)?,
        })
    }

    pub fn from_data(
        destination: String,
        destination_pos: (u16, u16),
        eta_month: u8,
        eta_day: u8,
        eta_hour: u8,
        eta_minute: u8,
        cargo_type: u8,
        speed_profile: u8,
    ) -> Self {
        Self {
            destination: destination,
            destination_position: destination_pos,
            eta_month: eta_month,
            eta_day: eta_day,
            eta_hour: eta_hour,
            eta_minute: eta_minute,
            cargo_type: cargo_type,
            speed_profile: speed_profile,
        }
    }

    pub fn to_bitpacker(&self) -> BitPacker {
        BitPacker::from_int(self.speed_profile, Some(8))
            + BitPacker::from_int(self.cargo_type, Some(8))
            + BitPacker::from_int(self.eta_minute, Some(8))
            + BitPacker::from_int(self.eta_hour, Some(8))
            + BitPacker::from_int(self.eta_day, Some(8))
            + BitPacker::from_int(self.eta_month, Some(8))
            + BitPacker::from_int(self.destination_position.1, Some(16))
            + BitPacker::from_int(self.destination_position.0, Some(16))
            + BitPacker::from_str(&self.destination, Some(120))
    }
}

impl VoyageOrder {
    pub fn from_bitpacker(voyage_order_bitpacker: BitPacker) -> VoyageOrderResult<Self> {
        Ok(Self {
            header: VoyageOrderHeader::from_bitpacker(
                voyage_order_bitpacker.slice(None, Some(23))?,
            )?,
            body: VoyageOrderBody::from_bitpacker(voyage_order_bitpacker.slice(Some(24), None)?)?,
        })
    }

    pub fn from_components(header: VoyageOrderHeader, body: VoyageOrderBody) -> Self {
        Self {
            header: header,
            body: body,
        }
    }

    pub fn header(&self) -> VoyageOrderHeader {
        self.header.clone()
    }

    pub fn body(&self) -> VoyageOrderBody {
        self.body.clone()
    }

    pub fn to_bitpacker(&self) -> BitPacker {
        self.body.to_bitpacker() + self.header.to_bitpacker()
    }

    pub fn is_rev_of(&self, order2: &VoyageOrder) -> bool {
        self.header.id == order2.header.id && self.header.version > order2.header.version
    }

    pub fn set_ver(&mut self, version: u8) -> () {
        self.header.set_version(version);
    }
}
