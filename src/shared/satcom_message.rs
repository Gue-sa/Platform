use crate::shared::{
    bitpacker::BitPacker,
    common::types::{SatComMessageResult, SatComMessageType},
    voyage_order::VoyageOrder,
};

#[derive(Debug, Clone)]
pub struct SatComMessage {
    pub source: u32,
    pub target: u32,
    pub order_id: u32,
    pub order_version: u8,
    pub message_type: SatComMessageType,
    pub order_review: Option<VoyageOrder>,
}

impl SatComMessage {
    pub fn new(
        source: u32,
        target: u32,
        order_id: u32,
        order_version: u8,
        message_type: SatComMessageType,
        order_review: Option<VoyageOrder>,
    ) -> Self {
        Self {
            source: source,
            target: target,
            order_id: order_id,
            order_version: order_version,
            message_type: message_type,
            order_review: order_review,
        }
    }

    pub fn parse(msg: BitPacker) -> SatComMessageResult<Self> {
        let mut order_review: Option<VoyageOrder> = None;

        if msg.bits_len > 111 {
            order_review = Some(VoyageOrder::from(msg.slice(Some(112), None)?)?);
        }

        Ok(Self {
            source: msg.extract_int::<u32>(None, Some(31))?,
            target: msg.extract_int::<u32>(Some(32), Some(63))?,
            order_id: msg.extract_int::<u32>(Some(64), Some(95))?,
            order_version: msg.extract_int::<u8>(Some(96), Some(103))?,
            message_type: msg.extract_int::<u8>(Some(104), Some(111))?.into(),
            order_review: order_review,
        })
    }

    pub fn to_bitpacker(&self) -> BitPacker {
        let order_review_bitpacker: BitPacker = if self.order_review.is_some() {
            self.order_review.as_ref().unwrap().to_bitpacker()
        } else {
            BitPacker::from_int(0, Some(0))
        };
        let u8_message_type: u8 = self.message_type.clone().into();

        order_review_bitpacker
            + BitPacker::from_int(u8_message_type, Some(8))
            + BitPacker::from_int(self.order_version, Some(8))
            + BitPacker::from_int(self.order_id, Some(32))
            + BitPacker::from_int(self.target, Some(32))
            + BitPacker::from_int(self.source, Some(32))
    }
}
