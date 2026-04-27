use crate::{
    bitpacker::BitPacker,
    boat_info::{BoatInfo, NavigationData, StaticData, VoyageData},
    common::{
        constants::{MSG5_FIELDS, MSG123_FIELDS, SOTDMA_CS_MSGS},
        types::{
            AisMessageError, AisMessageResult, CSType, CommunicationStateError,
            CommunicationStateResult,
        },
        utils::*,
    },
};
use chrono::Timelike;
use crc::{CRC_16_IBM_SDLC, Crc};
use getset::{CloneGetters, Getters, Setters};

#[derive(Clone, Debug, PartialEq)]
pub struct CommunicationState {
    communication_state_type: CSType,
    sync_state: u8,
    slot_timeout: Option<u8>,
    slot_offset: Option<u16>,
    utc_hour: Option<u8>,
    utc_minute: Option<u8>,
    slot_number: Option<u16>,
    received_stations: Option<u16>,
    slot_increment: Option<u16>,
    number_of_slots: Option<u8>,
    keep_flag: Option<bool>,
}

#[derive(Debug, Getters, Setters, CloneGetters)]
pub struct AisMessage {
    #[getset(get = "pub")]
    message_type: u8,
    #[getset(get_clone = "pub")]
    boat_info: BoatInfo,

    #[getset(get = "pub")]
    ramp_up_bits: BitPacker,
    #[getset(get = "pub")]
    sync_sequence: BitPacker,
    #[getset(get = "pub")]
    start_flag: BitPacker,
    #[getset(get = "pub")]
    data: BitPacker,
    communication_state: Option<CommunicationState>,
    #[getset(get = "pub")]
    crc: BitPacker,
    #[getset(get = "pub")]
    end_flag: BitPacker,
    #[getset(get = "pub")]
    buffer: BitPacker,
}

const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);

impl CommunicationState {
    pub fn init(
        msg_type: u8,
        sync_state: u8,
        slot_timeout: Option<u8>,
        slot_offset: Option<u16>,
        slot_nbr: Option<u16>,
        received_stations: Option<u16>,
        slot_increment: Option<u16>,
        nbr_of_slots: Option<u8>,
        keep_flag: Option<bool>,
    ) -> Self {
        CommunicationState {
            communication_state_type: if SOTDMA_CS_MSGS.binary_search(&msg_type).is_ok() {
                CSType::SOTDMA
            } else {
                CSType::ITDMA
            },
            sync_state: sync_state,
            slot_timeout: slot_timeout,
            slot_increment: slot_increment,
            slot_number: slot_nbr,
            slot_offset: slot_offset,
            number_of_slots: nbr_of_slots,
            received_stations: received_stations,
            keep_flag: keep_flag,
            utc_hour: Some(get_current_dt().hour() as u8),
            utc_minute: Some(get_current_dt().minute() as u8),
        }
    }

    pub fn communication_state_type(&self) -> CSType {
        self.communication_state_type
    }

    pub fn sync_state(&self) -> u8 {
        self.sync_state
    }

    pub fn slot_timeout(&self) -> CommunicationStateResult<&u8> {
        self.slot_timeout
            .as_ref()
            .ok_or_else(|| CommunicationStateError::NoSotdmaTimeout)
    }

    pub fn slot_offset(&self) -> CommunicationStateResult<&u16> {
        self.slot_offset
            .as_ref()
            .ok_or_else(|| CommunicationStateError::NoSotdmaSlotOffset)
    }

    pub fn utc_hour(&self) -> CommunicationStateResult<&u8> {
        self.utc_hour
            .as_ref()
            .ok_or_else(|| CommunicationStateError::NoUtcHour)
    }

    pub fn utc_minute(&self) -> CommunicationStateResult<&u8> {
        self.utc_minute
            .as_ref()
            .ok_or_else(|| CommunicationStateError::NoUtcMinute)
    }

    pub fn slot_number(&self) -> CommunicationStateResult<&u16> {
        self.slot_number
            .as_ref()
            .ok_or_else(|| CommunicationStateError::NoItdmaSlotNumber)
    }

    pub fn received_stations(&self) -> CommunicationStateResult<&u16> {
        self.received_stations
            .as_ref()
            .ok_or_else(|| CommunicationStateError::NoSotdmaReceivedStations)
    }

    pub fn slot_increment(&self) -> CommunicationStateResult<&u16> {
        self.slot_increment
            .as_ref()
            .ok_or_else(|| CommunicationStateError::NoItdmaSlotIncrement)
    }

    pub fn number_of_slots(&self) -> CommunicationStateResult<&u8> {
        self.number_of_slots
            .as_ref()
            .ok_or_else(|| CommunicationStateError::NoItdmaNumberOfSlots)
    }

    pub fn keep_flag(&self) -> CommunicationStateResult<&bool> {
        self.keep_flag
            .as_ref()
            .ok_or_else(|| CommunicationStateError::NoItdmaKeepFlag)
    }

    pub fn parse(com_state_bitpacker: BitPacker, msg_type: u8) -> CommunicationStateResult<Self> {
        let mut com_state: Self = Self {
            communication_state_type: if SOTDMA_CS_MSGS.binary_search(&msg_type).is_ok() {
                CSType::SOTDMA
            } else {
                CSType::ITDMA
            },
            sync_state: com_state_bitpacker.extract_int(None, Some(1))?,
            slot_timeout: None,
            slot_offset: None,
            utc_hour: None,
            utc_minute: None,
            slot_number: None,
            received_stations: None,
            slot_increment: None,
            number_of_slots: None,
            keep_flag: None,
        };

        match msg_type {
            1 | 2 => {
                let slot_timeout: u8 = com_state_bitpacker.extract_int::<u8>(Some(2), Some(4))?;
                let sub_msg: BitPacker = com_state_bitpacker.slice(Some(5), None)?;

                com_state.slot_timeout = Some(slot_timeout);

                match slot_timeout {
                    0 => {
                        com_state.slot_offset = Some(sub_msg.extract_int::<u16>(None, None)?);
                    }
                    1 => {
                        com_state.utc_hour = Some(sub_msg.extract_int::<u8>(None, Some(7))?);
                        com_state.utc_minute = Some(sub_msg.extract_int::<u8>(Some(8), None)?);
                    }
                    2 | 4 | 6 => {
                        com_state.slot_number = Some(sub_msg.extract_int::<u16>(None, None)?);
                    }
                    3 | 5 | 7 => {
                        com_state.received_stations = Some(sub_msg.extract_int::<u16>(None, None)?);
                    }
                    _ => return Err(CommunicationStateError::UnkownSotdmaTimeout),
                }
            }
            3 => {
                com_state.slot_increment =
                    Some(com_state_bitpacker.extract_int::<u16>(Some(2), Some(14))?);
                com_state.number_of_slots =
                    Some(com_state_bitpacker.extract_int::<u8>(Some(15), Some(17))?);
                com_state.keep_flag =
                    if com_state_bitpacker.extract_int::<u8>(Some(18), Some(18))? == 1 {
                        Some(true)
                    } else {
                        Some(false)
                    };
            }
            _ => return Err(CommunicationStateError::UnknownMessageType),
        }

        Ok(com_state)
    }

    fn build_sub_message(&self) -> CommunicationStateResult<BitPacker> {
        if *self.slot_timeout()? == 3 || *self.slot_timeout()? == 5 || *self.slot_timeout()? == 7 {
            Ok(BitPacker::from_int(*self.received_stations()?, Some(14)))
        } else if *self.slot_timeout()? == 2
            || *self.slot_timeout()? == 4
            || *self.slot_timeout()? == 6
        {
            Ok(BitPacker::from_int(*self.slot_number()?, Some(14)))
        } else if *self.slot_timeout()? == 1 {
            Ok(BitPacker::from_int(0, Some(3))
                + BitPacker::from_int(get_current_dt().hour(), Some(5))
                + BitPacker::from_int(get_current_dt().minute(), Some(6)))
        } else {
            Ok(BitPacker::from_int(*self.slot_offset()?, Some(14)))
        }
    }

    pub fn build(&self) -> CommunicationStateResult<BitPacker> {
        match self.communication_state_type {
            CSType::SOTDMA => Ok(self.build_sub_message()?
                + BitPacker::from_int(*self.slot_timeout()?, Some(3))
                + BitPacker::from_int(self.sync_state, Some(2))),
            CSType::ITDMA => Ok(BitPacker::from_int(
                if *self.keep_flag()? { 1 } else { 0 },
                Some(1),
            ) + BitPacker::from_int(*self.number_of_slots()?, Some(3))
                + BitPacker::from_int(*self.slot_increment()?, Some(13))
                + BitPacker::from_int(self.sync_state, Some(2))),
        }
    }
}

impl AisMessage {
    fn compute_crc(bytes: &[u8]) -> Result<u16, &'static str> {
        Ok(X25.checksum(bytes))
    }

    pub fn communication_state(&self) -> AisMessageResult<CommunicationState> {
        self.communication_state
            .clone()
            .ok_or(AisMessageError::NoCommunicationState)
    }

    pub fn parse(
        msg: BitPacker,
    ) -> AisMessageResult<(u8, BitPacker, Option<CommunicationState>, u16, BoatInfo)> {
        let msg_type: u8 = msg.extract_int::<u8>(Some(40), Some(45))?;

        let mut static_data: StaticData = StaticData::new(
            None, None, None, None, None, None, None, None, None, None, None, None,
        );

        let mut voyage_data: VoyageData =
            VoyageData::new(None, None, None, None, None, None, None, None);

        let mut navigation_data: NavigationData =
            NavigationData::new(None, None, None, None, None, None, None, None, None);

        match msg_type {
            1 | 2 | 3 => {
                let payload: BitPacker = msg.slice(Some(40), Some(207))?;
                let data: BitPacker = payload.slice(Some(8), Some(148))?;
                let communication_state: CommunicationState =
                    CommunicationState::parse(payload.slice(Some(149), Some(167))?, msg_type)?;
                let msg_crc: u16 = msg.extract_int::<u16>(Some(208), Some(223))?;
                let computed_crc: u16 = AisMessage::compute_crc(payload.bits()).unwrap();

                if msg_crc == computed_crc {
                    static_data.set_mmsi(data.extract_int::<u32>(None, Some(29))?);
                    static_data.set_position_accuracy(data.extract_int::<u8>(Some(52), Some(52))?);

                    voyage_data.set_raim_flag(data.extract_int::<u8>(Some(140), Some(140))?);

                    navigation_data
                        .set_navigational_status(data.extract_int::<u8>(Some(30), Some(33))?);
                    navigation_data.set_time_stamp(data.extract_int::<u8>(Some(129), Some(134))?);
                    navigation_data.set_special_maneuvre_indicator(
                        data.extract_int::<u8>(Some(135), Some(136))?,
                    );
                    navigation_data.set_latitude(data.extract_int::<u32>(Some(53), Some(80))?);
                    navigation_data.set_longitude(data.extract_int::<u32>(Some(81), Some(107))?);
                    navigation_data
                        .set_course_over_ground(data.extract_int::<u16>(Some(108), Some(119))?);
                    navigation_data
                        .set_speed_over_ground(data.extract_int::<u16>(Some(42), Some(51))?);
                    navigation_data.set_rate_of_turn(data.extract_int::<i8>(Some(34), Some(41))?);
                    navigation_data
                        .set_true_heading(data.extract_int::<u16>(Some(120), Some(128))?);

                    let boat_info: BoatInfo =
                        BoatInfo::new(Some(static_data), Some(voyage_data), Some(navigation_data));

                    Ok((
                        msg_type,
                        data,
                        Some(communication_state),
                        msg_crc,
                        boat_info,
                    ))
                } else {
                    return Err(AisMessageError::CrcMismatch);
                }
            }
            5 => {
                let payload: BitPacker = msg.slice(Some(40), Some(463))?;
                let data: BitPacker = payload.slice(Some(8), None)?;
                let msg_crc: u16 = msg.slice(Some(464), Some(479))?.extract_int(None, None)?;
                let computed_crc: u16 = AisMessage::compute_crc(&payload.bits()).unwrap();

                if msg_crc == computed_crc {
                    static_data.set_mmsi(data.extract_int::<u32>(None, Some(29))?);
                    static_data.set_imo_number(data.extract_int::<u32>(Some(32), Some(61))?);
                    static_data.set_call_sign(data.extract_str(Some(62), Some(103))?);
                    static_data.set_name(data.extract_str(Some(104), Some(223))?);
                    static_data.set_type_of_ship_and_cargo_type(
                        data.extract_int::<u8>(Some(224), Some(231))?,
                    );
                    static_data.set_ais_version(data.extract_int::<u8>(Some(30), Some(31))?);
                    static_data
                        .set_type_of_epf_device(data.extract_int::<u8>(Some(262), Some(265))?);
                    static_data.set_a(data.extract_int::<u16>(Some(232), Some(240))?);
                    static_data.set_b(data.extract_int::<u16>(Some(241), Some(249))?);
                    static_data.set_c(data.extract_int::<u8>(Some(250), Some(255))?);
                    static_data.set_d(data.extract_int::<u8>(Some(256), Some(261))?);

                    voyage_data.set_destination(data.extract_str(Some(294), Some(413))?);
                    voyage_data.set_eta_month(data.extract_int::<u8>(Some(282), Some(285))?);
                    voyage_data.set_eta_day(data.extract_int::<u8>(Some(277), Some(281))?);
                    voyage_data.set_eta_hour(data.extract_int::<u8>(Some(272), Some(276))?);
                    voyage_data.set_eta_minute(data.extract_int::<u8>(Some(266), Some(271))?);
                    voyage_data.set_maximum_present_static_draught(
                        data.extract_int::<u8>(Some(286), Some(293))?,
                    );
                    voyage_data.set_dte(data.extract_int::<u8>(Some(414), Some(414))?);

                    let boat_info: BoatInfo =
                        BoatInfo::new(Some(static_data), Some(voyage_data), None);

                    Ok((msg_type, data, None, msg_crc, boat_info))
                } else {
                    Err(AisMessageError::CrcMismatch)
                }
            }
            _ => Err(AisMessageError::UnknownMessageType),
        }
    }

    pub fn from_bits(msg: BitPacker) -> AisMessageResult<Self> {
        let (message_type, data, communication_state, crc, boat_info) = AisMessage::parse(msg)?;

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
            buffer: BitPacker::from_int::<u32>(8388607, Some(23)),
        })
    }

    pub fn from_info(
        boat_info: BoatInfo,
        message_type: u8,
        communication_state: Option<CommunicationState>,
    ) -> AisMessageResult<Self> {
        let data: BitPacker = AisMessage::build_data_bytes(&boat_info, message_type);
        let crc: u16 = AisMessage::compute_crc(
            AisMessage::build_payload(message_type, data.clone(), communication_state.clone())?
                .bits(),
        )
        .unwrap();

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
            buffer: BitPacker::from_int::<u32>(8388607, Some(23)),
        })
    }

    fn build_data_bytes(boat_info: &BoatInfo, msg_type: u8) -> BitPacker {
        let mut data_vec: BitPacker = BitPacker::from_int(0, Some(0));

        match msg_type {
            1 | 2 | 3 => {
                for field in MSG123_FIELDS {
                    data_vec = boat_info.to_bits(field, msg_type) + data_vec;
                }
            }
            5 => {
                for field in MSG5_FIELDS {
                    data_vec = boat_info.to_bits(field, msg_type) + data_vec;
                }
            }
            _ => {}
        }

        data_vec
    }

    fn build_payload(
        msg_type: u8,
        data: BitPacker,
        communication_state: Option<CommunicationState>,
    ) -> AisMessageResult<BitPacker> {
        if communication_state.is_none() {
            Ok(data
                + BitPacker::from_int::<u8>(3, Some(2))
                + BitPacker::from_int::<u8>(msg_type, Some(6)))
        } else {
            Ok(communication_state.unwrap().build()?
                + data
                + BitPacker::from_int::<u8>(3, Some(2))
                + BitPacker::from_int::<u8>(msg_type, Some(6)))
        }
    }

    pub fn build(&self) -> AisMessageResult<BitPacker> {
        let payload: BitPacker = AisMessage::build_payload(
            self.message_type,
            AisMessage::build_data_bytes(&self.boat_info, self.message_type),
            self.communication_state.clone(),
        )?;
        let crc: BitPacker =
            BitPacker::from_int::<u16>(AisMessage::compute_crc(payload.bits()).unwrap(), Some(16));
        let msg: BitPacker = self.buffer.clone()
            + self.end_flag.clone()
            + crc
            + payload
            + self.start_flag.clone()
            + self.sync_sequence.clone()
            + self.ramp_up_bits.clone();

        Ok(msg)
    }
}
