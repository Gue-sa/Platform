use chrono::Timelike;
use crc::{CRC_16_IBM_SDLC, Crc};

use crate::{boat_info::{BoatInfo, NavigationData, StaticData, VoyageData}, common::{constants::{IMPLEMENTED_MSGS, MSG5_FIELDS, MSG123_FIELDS, NO_CS_MSGS, SOTDMA_CS_MSGS}, types::*, utils::*}, shared::bitpacker::BitPacker};


#[derive(Clone, Debug)]
pub struct CommunicationState {
    pub cstype: CSTypes,
    pub sync_state: u8,
    pub slot_timeout: Option<u8>,
    pub slot_offset: Option<u16>,
    pub utc_hour: Option<u8>,
    pub utc_minute: Option<u8>,
    pub slot_number: Option<u16>,
    pub received_stations: Option<u16>,
    pub slot_increment: Option<u16>,
    pub number_of_slots: Option<u8>,
    pub keep_flag: Option<bool>
}


#[derive(Debug)]
pub struct Message {
    pub message_type: u8,
    pub boat_info: BoatInfo,

    pub ramp_up_bits: BitPacker,
    pub sync_sequence: BitPacker,
    pub start_flag: BitPacker,
    pub data: BitPacker,
    pub communication_state: Option<CommunicationState>,
    pub crc: BitPacker,
    pub end_flag: BitPacker,
    pub buffer: BitPacker
}

const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);


impl CommunicationState {
    pub fn init(msg_type: u8,
                sync_state: u8,
                slot_timeout: Option<u8>,
                slot_offset: Option<u16>,
                slot_nbr: Option<u16>,
                received_stations: Option<u16>,
                slot_increment: Option<u16>,
                number_of_slots: Option<u8>,
                keep_flag: Option<bool>) -> Self {
        
        CommunicationState {
            cstype: if SOTDMA_CS_MSGS.binary_search(&msg_type).is_ok() {CSTypes::SOTDMA} else {CSTypes::ITDMA},
            sync_state: sync_state,
            slot_timeout: slot_timeout,
            slot_increment: slot_increment,
            slot_number: slot_nbr,
            slot_offset: slot_offset,
            number_of_slots: number_of_slots,
            received_stations: received_stations,
            keep_flag: keep_flag,
            utc_hour: Some(get_current_datetime().hour() as u8),
            utc_minute: Some(get_current_datetime().minute() as u8)
        }
    }

    
    pub fn parse(communication_state: BitPacker, message_type: u8) -> MessageResult<Self> {
        let mut cs: Self = Self {
            cstype: if SOTDMA_CS_MSGS.binary_search(&message_type).is_ok() {CSTypes::SOTDMA} else {CSTypes::ITDMA},
            sync_state: communication_state.extract_int(None, Some(1))?,
            slot_timeout: None,
            slot_offset: None,
            utc_hour: None,
            utc_minute: None,
            slot_number: None,
            received_stations: None,
            slot_increment: None,
            number_of_slots: None,
            keep_flag: None
        };

        match message_type {
            1 | 2 => {
                let slot_timeout: u8 = communication_state.extract_int::<u8>(Some(2), Some(4))?;
                let sub_message: BitPacker = communication_state.slice(Some(5), None)?;

                cs.slot_timeout = Some(slot_timeout);

                match slot_timeout {
                    0 => {
                        cs.slot_offset = Some(sub_message.extract_int::<u16>(None, None)?);
                    },
                    1 => {
                        cs.utc_hour = Some(sub_message.extract_int::<u8>(None, Some(7))?);
                        cs.utc_minute = Some(sub_message.extract_int::<u8>(Some(8), None)?);
                    },
                    2 | 4 | 6 => {
                        cs.slot_number = Some(sub_message.extract_int::<u16>(None, None)?);
                    },
                    3 | 5 | 7 => {
                        cs.received_stations = Some(sub_message.extract_int::<u16>(None, None)?);
                    },
                    _ => {}
                }
            },
            3 => {
                cs.slot_increment = Some(communication_state.extract_int::<u16>(Some(2), Some(14))?);
                cs.number_of_slots = Some(communication_state.extract_int::<u8>(Some(15), Some(17))?);
                cs.keep_flag = if communication_state.extract_int::<u8>(Some(18), Some(18))? == 1 {Some(true)} else {Some(false)};
            },
            _ => {}
        }

        Ok(cs)
    }


    pub fn build_sub_message(&self) -> BitPacker {
        if self.slot_timeout.unwrap() == 3 || self.slot_timeout.unwrap() == 5 || self.slot_timeout.unwrap() == 7 {
            BitPacker::from_int(self.received_stations.unwrap(), Some(14))
        } else if self.slot_timeout.unwrap() == 2 || self.slot_timeout.unwrap() == 4 || self.slot_timeout.unwrap() == 6 {
            BitPacker::from_int(self.slot_number.unwrap(), Some(14))
        } else if self.slot_timeout.unwrap() == 1 {
            BitPacker::from_int(0, Some(3))
            + BitPacker::from_int(get_current_datetime().hour(), Some(5))
            + BitPacker::from_int(get_current_datetime().minute(), Some(6))
        } else {
            BitPacker::from_int(self.slot_offset.unwrap(), Some(14))
        }
    }


    pub fn build(&self) -> BitPacker {
        match self.cstype {
            CSTypes::SOTDMA => {
                self.build_sub_message() +
                BitPacker::from_int(self.slot_timeout.unwrap(), Some(3)) +
                BitPacker::from_int(self.sync_state, Some(2))
            },
            CSTypes::ITDMA => {
                BitPacker::from_int(if self.keep_flag.unwrap() {1} else {0}, Some(1)) +
                BitPacker::from_int(self.number_of_slots.unwrap(), Some(3)) +
                BitPacker::from_int(self.slot_increment.unwrap(), Some(13)) +
                BitPacker::from_int(self.sync_state, Some(2))
            }
        }
    }
}


impl Message {
    pub fn compute_crc(bytes: &[u8]) -> Result<u16, &'static str> {
        Ok(X25.checksum(bytes))
    }


    pub fn parse(msg: BitPacker) -> MessageResult<(u8, BitPacker, Option<CommunicationState>, u16, BoatInfo)> {
        let msg_type: u8 = msg.extract_int::<u8>(Some(40), Some(45))?;

        let mut static_data: StaticData = StaticData::init(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None);

        let mut voyage_data: VoyageData = VoyageData::init(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None);

            let mut navigation_data: NavigationData = NavigationData::init(
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None);

        match msg_type {
            1 | 2 | 3 => {
                let payload: BitPacker = msg.slice(Some(40), Some(207))?;
                let data: BitPacker = payload.slice(Some(8), Some(148))?;
                let communication_state: CommunicationState = CommunicationState::parse(payload.slice(Some(149), Some(167))?, msg_type)?;
                let msg_crc: u16 = msg.extract_int::<u16>(Some(208), Some(223))?;
                let computed_crc: u16 = Message::compute_crc(payload.bits()).unwrap();

                if msg_crc == computed_crc {
                    static_data.mmsi = data.extract_int::<u32>(None, Some(29))?;
                    static_data.position_accuracy = data.extract_int::<u8>(Some(52), Some(52))?;

                    voyage_data.raim_flag = data.extract_int::<u8>(Some(140), Some(140))?;

                    navigation_data.navigational_status = data.extract_int::<u8>(Some(30), Some(33))?;
                    navigation_data.time_stamp = data.extract_int::<u8>(Some(129), Some(134))?;
                    navigation_data.special_maneuvre_indicator = data.extract_int::<u8>(Some(135), Some(136))?;
                    navigation_data.latitude = data.extract_int::<u32>(Some(53), Some(80))?;
                    navigation_data.longitude = data.extract_int::<u32>(Some(81), Some(107))?;
                    navigation_data.course_over_ground = data.extract_int::<u16>(Some(108), Some(119))?;
                    navigation_data.speed_over_ground = data.extract_int::<u16>(Some(42), Some(51))?;
                    navigation_data.rate_of_turn = data.extract_int::<i8>(Some(34), Some(41))?;
                    navigation_data.true_heading = data.extract_int::<u16>(Some(120), Some(128))?;

                    let boat_info: BoatInfo = BoatInfo::init(Some(static_data), Some(voyage_data), Some(navigation_data));

                    Ok((msg_type, data, Some(communication_state), msg_crc, boat_info))
                } else {
                    return Err(MessageError::CrcMismatch)
                }
            },
            5 => {
                let payload: BitPacker = msg.slice(Some(40), Some(463))?;
                let data: BitPacker = payload.slice(Some(8), None)?;
                let msg_crc: u16 = msg.slice(Some(464), Some(479))?.extract_int(None, None)?;
                let computed_crc: u16 = Message::compute_crc(payload.bits()).unwrap();

                if msg_crc == computed_crc {
                    static_data.mmsi = data.extract_int::<u32>(None, Some(29))?;
                    static_data.imo_number = data.extract_int::<u32>(Some(32), Some(61))?;
                    static_data.call_sign = data.extract_str(Some(62), Some(103))?;
                    static_data.name = data.extract_str(Some(104), Some(223))?;
                    static_data.type_of_ship_and_cargo_type = data.extract_int::<u8>(Some(224), Some(231))?;
                    static_data.ais_version = data.extract_int::<u8>(Some(30), Some(31))?;
                    static_data.type_of_epf_device = data.extract_int::<u8>(Some(262), Some(265))?;
                    static_data.a = data.extract_int::<u16>(Some(232), Some(240))?;
                    static_data.b = data.extract_int::<u16>(Some(241), Some(249))?;
                    static_data.c = data.extract_int::<u8>(Some(250), Some(255))?;
                    static_data.d = data.extract_int::<u8>(Some(256), Some(261))?;

                    voyage_data.destination = data.extract_str(Some(294), Some(413))?;
                    voyage_data.eta_month = data.extract_int::<u8>(Some(282), Some(285))?;
                    voyage_data.eta_day = data.extract_int::<u8>(Some(277), Some(281))?;
                    voyage_data.eta_hour = data.extract_int::<u8>(Some(272), Some(276))?;
                    voyage_data.eta_minute = data.extract_int::<u8>(Some(266), Some(271))?;
                    voyage_data.maximum_present_static_draught = data.extract_int::<u8>(Some(286), Some(293))?;
                    voyage_data.dte = data.extract_int::<u8>(Some(414), Some(414))?;
                    
                    let boat_info: BoatInfo = BoatInfo::init(Some(static_data), Some(voyage_data), None);

                    Ok((msg_type, data, None, msg_crc, boat_info))
                } else {
                    Err(MessageError::CrcMismatch)
                }
            },
            _ => Err(MessageError::UnknownMessageType)
        }
    }


    pub fn from_bits(msg: BitPacker) -> MessageResult<Self> {
        let (message_type, data, communication_state, crc, boat_info) = Message::parse(msg)?;

        Ok(Self {
            message_type: message_type,
            boat_info: boat_info,

            ramp_up_bits: BitPacker::from_int::<u8>(255, Some(8)),
            sync_sequence: BitPacker::from_int::<u32>(5592405, Some(24)),
            start_flag: BitPacker::from_int::<u8>(126, Some(8)),
            data: data,
            communication_state: communication_state,
            crc: BitPacker::from_int(crc, Some(16)),
            end_flag: BitPacker::from_int::<u8>(126, Some(8)),
            buffer: BitPacker::from_int::<u32>(8388607, Some(23))
        })
    }


    pub fn from_info(boat_info: BoatInfo, message_type: u8, communication_state: Option<CommunicationState>) -> Self {
        let data: BitPacker = Message::build_data_bytes(&boat_info, message_type);
        let crc: u16 = Message::compute_crc(Message::build_payload(message_type, data.clone(), communication_state.clone()).bits()).unwrap();

        Self {
            message_type: message_type,
            boat_info: boat_info,

            ramp_up_bits: BitPacker::from_int::<u8>(255, Some(8)),
            sync_sequence: BitPacker::from_int::<u32>(5592405, Some(24)),
            start_flag: BitPacker::from_int::<u8>(126, Some(8)),
            data: data,
            communication_state: communication_state,
            crc: BitPacker::from_int(crc, Some(16)),
            end_flag: BitPacker::from_int::<u8>(126, Some(8)),
            buffer: BitPacker::from_int::<u32>(8388607, Some(23))
        }
    }


    pub fn build_data_bytes(boat_info: &BoatInfo, msg_type: u8) -> BitPacker {
        let mut data_vec: BitPacker = BitPacker::from_int(0, Some(0));

        match msg_type {
            1 | 2 | 3 => {
                for field in MSG123_FIELDS {
                    data_vec = boat_info.get_as_bits(field, msg_type) + data_vec;
                }
            },
            5 => {
                for field in MSG5_FIELDS {
                    data_vec = boat_info.get_as_bits(field, msg_type) + data_vec;
                }
            },
            _ => {}
        }

        data_vec
    }


    pub fn build_payload(msg_type: u8, data: BitPacker, communication_state: Option<CommunicationState>) -> BitPacker {
        if communication_state.is_none() {
            data +
            BitPacker::from_int::<u8>(3, Some(2)) +
            BitPacker::from_int::<u8>(msg_type, Some(6))
        } else {
            communication_state.unwrap().build() +
            data +
            BitPacker::from_int::<u8>(3, Some(2)) +
            BitPacker::from_int::<u8>(msg_type, Some(6))
        }
    }


    pub fn build(&self) -> BitPacker {
        let payload: BitPacker = Message::build_payload(self.message_type, Message::build_data_bytes(&self.boat_info, self.message_type), self.communication_state.clone());
        let crc: BitPacker = BitPacker::from_int::<u16>(Message::compute_crc(payload.bits()).unwrap(), Some(16));
        let msg: BitPacker = self.buffer.clone() + self.end_flag.clone() + crc + payload + self.start_flag.clone() + self.sync_sequence.clone() + self.ramp_up_bits.clone();
        
        msg
    }
}