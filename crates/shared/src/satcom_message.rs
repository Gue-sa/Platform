use crate::{
    bitpacker::BitPacker,
    common::types::{SatComMessageResult, SatComMessageType},
    voyage_order::{VoyageOrder, VoyageOrderBody, VoyageOrderHeader},
};

#[derive(Debug, Clone)]
pub struct SatComMessage {
    pub source: u32,
    pub target: u32,
    pub order_header: VoyageOrderHeader,
    pub message_type: SatComMessageType,
    pub order_body_review: Option<VoyageOrderBody>,
}

impl SatComMessage {
    pub fn new(
        source: u32,
        target: u32,
        order_header: VoyageOrderHeader,
        message_type: SatComMessageType,
        order_body_review: Option<VoyageOrderBody>,
    ) -> Self {
        Self {
            source: source,
            target: target,
            order_header: order_header,
            message_type: message_type,
            order_body_review,
        }
    }

    pub fn parse(msg: BitPacker) -> SatComMessageResult<Self> {
        let mut order_body_review: Option<VoyageOrderBody> = None;

        if msg.bits_len > 112 {
            order_body_review = Some(VoyageOrderBody::from(msg.slice(Some(112), None)?)?);
        }

        Ok(Self {
            source: msg.extract_int::<u32>(None, Some(31))?,
            target: msg.extract_int::<u32>(Some(32), Some(63))?,
            order_header: VoyageOrderHeader::from(msg.slice(Some(64), Some(103))?)?,
            message_type: msg.extract_int::<u8>(Some(104), Some(111))?.into(),
            order_body_review,
        })
    }

    pub fn to_bitpacker(&self) -> BitPacker {
        let order_body_review_bitpacker: BitPacker = if self.order_body_review.is_some() {
            self.order_body_review.as_ref().unwrap().to_bitpacker()
        } else {
            BitPacker::from_int(0, Some(0))
        };
        let u8_message_type: u8 = self.message_type.clone().into();

        order_body_review_bitpacker
            + BitPacker::from_int(u8_message_type, Some(8))
            + self.order_header.to_bitpacker()
            + BitPacker::from_int(self.target, Some(32))
            + BitPacker::from_int(self.source, Some(32))
    }
}
