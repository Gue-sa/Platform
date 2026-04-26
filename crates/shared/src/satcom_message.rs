use crate::{
    bitpacker::BitPacker,
    common::types::{SatComMessageResult, SatComMessageType},
    voyage_order::{VoyageOrder, VoyageOrderBody, VoyageOrderHeader},
};

use getset::{CloneGetters, Getters, Setters};

#[derive(Debug, Clone, Getters, Setters, CloneGetters)]
pub struct SatComMessage {
    #[getset(get = "pub")]
    source: u32,
    #[getset(get = "pub")]
    target: u32,
    #[getset(get_clone = "pub")]
    order_header: VoyageOrderHeader,
    #[getset(get = "pub")]
    message_type: SatComMessageType,
    #[getset(get_clone = "pub")]
    order_body_review: Option<VoyageOrderBody>,
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

    pub fn order(&self) -> Option<VoyageOrder> {
        if self.order_body_review().is_some() {
            Some(VoyageOrder::from(
                self.order_header(),
                self.order_body_review().unwrap(),
            ))
        } else {
            None
        }
    }

    pub fn parse(msg: BitPacker) -> SatComMessageResult<Self> {
        let mut order_body_review: Option<VoyageOrderBody> = None;

        if *msg.bits_len() > 112 {
            order_body_review = Some(VoyageOrderBody::from_bitpacker(
                msg.slice(Some(112), None)?,
            )?);
        }

        Ok(Self {
            source: msg.extract_int::<u32>(None, Some(31))?,
            target: msg.extract_int::<u32>(Some(32), Some(63))?,
            order_header: VoyageOrderHeader::from_bitpacker(msg.slice(Some(64), Some(103))?)?,
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
