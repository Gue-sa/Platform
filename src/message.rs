use std::sync::{RwLock, atomic::AtomicU32};

use chrono::Timelike;
use crc::{CRC_16_IBM_SDLC, Crc};

use crate::{boat_info::BoatInfo, common::{types::*, utils::*}, impl_option_access};


#[derive(Clone, Debug)]
pub struct CommunicationState {
    pub cstype: CSTypes,
    pub sync_state: u8,
    pub slot_timeout: Option<u8>,
    pub slot_offset: Option<u16>,
    pub utc_hour: Option<u8>,
    pub utc_minute: Option<u8>,
    pub slot_number: Option<u16>,
    pub received_stations: Option<u8>,
    pub slot_increment: Option<u16>,
    pub number_of_slots: Option<u8>,
    pub keep_flag: Option<bool>
}


#[derive(Debug)]
pub struct Message {
    pub message_type: u8,
    pub boat_info: BoatInfo,

    pub ramp_up_bits: String,
    pub sync_sequence: String,
    pub start_flag: String,
    pub data: String,
    pub communication_state: Option<CommunicationState>,
    pub crc: String,
    pub end_flag: String,
    pub buffer: String
}

const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);

const MSG123_FIELDS: [&str; 13] = [
    "mmsi",
    "navigational_status",
    "rate_of_turn",
    "speed_over_ground",
    "position_accuracy",
    "longitude",
    "latitude",
    "course_over_ground",
    "true_heading",
    "time_stamp",
    "special_maneuvre_indicator",
    "spare",
    "raim_flag"
];

const MSG5_FIELDS: [&str; 19] = [
    "mmsi",
    "ais_version",
    "imo_number",
    "call_sign",
    "name",
    "type_of_ship_and_cargo_type",
    "a",
    "b",
    "c",
    "d",
    "type_of_epf_device",
    "eta_minute",
    "eta_hour",
    "eta_day",
    "eta_month",
    "maximum_present_static_draught",
    "destination",
    "dte",
    "spare"
];


impl CommunicationState {
    pub fn init(msg_type: u8, sync_state: Option<u8>, slot_timeout: Option<u8>, slot_offset: Option<u16>, slot_nbr: Option<u16>, received_stations: Option<u8>, slot_increment: Option<u16>, number_of_slots: Option<u8>, keep_flag: Option<bool>) -> Self {
        match msg_type {
            1 | 2 => {
                match slot_timeout.unwrap() {
                    0 => Self {
                        cstype: CSTypes::SOTDMA,
                        sync_state: sync_state.unwrap(),
                        slot_timeout: slot_timeout,
                        slot_offset: slot_offset,
                        utc_hour: None,
                        utc_minute: None,
                        slot_number: None,
                        received_stations: None,
                        slot_increment: None,
                        number_of_slots: None,
                        keep_flag: None
                    },
                    1 => Self {
                        cstype: CSTypes::SOTDMA,
                        sync_state: sync_state.unwrap(),
                        slot_timeout: slot_timeout,
                        slot_offset: None,
                        utc_hour: Some(get_current_datetime().hour() as u8),
                        utc_minute: Some(get_current_datetime().minute() as u8),
                        slot_number: None,
                        received_stations: None,
                        slot_increment: None,
                        number_of_slots: None,
                        keep_flag: None
                    },
                    2 | 4 | 6 => Self {
                        cstype: CSTypes::SOTDMA,
                        sync_state: sync_state.unwrap(),
                        slot_timeout: slot_timeout,
                        slot_offset: None,
                        utc_hour: None,
                        utc_minute: None,
                        slot_number: slot_nbr,
                        received_stations: None,
                        slot_increment: None,
                        number_of_slots: None,
                        keep_flag: None
                    },
                    3 | 5 | 7 => Self {
                        cstype: CSTypes::SOTDMA,
                        sync_state: sync_state.unwrap(),
                        slot_timeout: slot_timeout,
                        slot_offset: None,
                        utc_hour: None,
                        utc_minute: None,
                        slot_number: None,
                        received_stations: received_stations,
                        slot_increment: None,
                        number_of_slots: None,
                        keep_flag: None
                    },
                    _ => {panic!("Erreur : timeout de {} illégal.", slot_timeout.unwrap())}
                }
            },
            3 => Self {
                cstype: CSTypes::ITDMA,
                sync_state: sync_state.unwrap(),
                slot_timeout: None,
                slot_offset: None,
                utc_hour: None,
                utc_minute: None,
                slot_number: None,
                received_stations: None,
                slot_increment: slot_increment,
                number_of_slots: number_of_slots,
                keep_flag: keep_flag
            },
            _ => todo!()
        }
    }

    impl_option_access!(slot_timeout, u8, slot_timeout, set_slot_timeout);
    impl_option_access!(slot_offset, u16, slot_offset, set_slot_offset);
    impl_option_access!(utc_hour, u8, utc_hour, set_utc_hour);
    impl_option_access!(utc_minute, u8, utc_minute, set_utc_minute);
    impl_option_access!(slot_number, u16, slot_number, set_slot_number);
    impl_option_access!(received_stations, u8, received_stations, set_received_stations);
    impl_option_access!(slot_increment, u16, slot_increment, set_slot_increment);
    impl_option_access!(number_of_slots, u8, number_of_slots, set_number_of_slots);
    impl_option_access!(keep_flag, bool, keep_flag, set_keep_flag);

    pub fn parse(communication_state: &str, message_type: u8) -> Result<Self, &'static str> {
        match message_type {
            1 | 2 => {
                let slot_timeout: u8 = u8::from_str_radix(&communication_state[2..5], 2).unwrap();
                let sub_message: &str = &communication_state[5..];

                match slot_timeout {
                    0 => Ok(Self {
                        cstype: CSTypes::SOTDMA,
                        sync_state: u8::from_str_radix(&communication_state[0..2], 2).unwrap(),
                        slot_timeout: Some(slot_timeout),
                        slot_offset: Some(u16::from_str_radix(sub_message, 2).unwrap()),
                        utc_hour: None,
                        utc_minute: None,
                        slot_number: None,
                        received_stations: None,
                        slot_increment: None,
                        number_of_slots: None,
                        keep_flag: None
                    }),
                    1 => Ok(Self {
                        cstype: CSTypes::SOTDMA,
                        sync_state: u8::from_str_radix(&communication_state[0..2], 2).unwrap(),
                        slot_timeout: Some(slot_timeout),
                        slot_offset: None,
                        utc_hour: Some(u8::from_str_radix(&sub_message[..8], 2).unwrap()),
                        utc_minute: Some(u8::from_str_radix(&sub_message[8..], 2).unwrap()),
                        slot_number: None,
                        received_stations: None,
                        slot_increment: None,
                        number_of_slots: None,
                        keep_flag: None
                    }),
                    2 | 4 | 6 => Ok(Self {
                        cstype: CSTypes::SOTDMA,
                        sync_state: u8::from_str_radix(&communication_state[0..2], 2).unwrap(),
                        slot_timeout: Some(slot_timeout),
                        slot_offset: None,
                        utc_hour: None,
                        utc_minute: None,
                        slot_number: Some(u16::from_str_radix(sub_message, 2).unwrap()),
                        received_stations: None,
                        slot_increment: None,
                        number_of_slots: None,
                        keep_flag: None
                    }),
                    3 | 5 | 7 => Ok(Self {
                        cstype: CSTypes::SOTDMA,
                        sync_state: u8::from_str_radix(&communication_state[0..2], 2).unwrap(),
                        slot_timeout: Some(slot_timeout),
                        slot_offset: None,
                        utc_hour: None,
                        utc_minute: None,
                        slot_number: None,
                        received_stations: Some(u8::from_str_radix(sub_message, 2).unwrap()),
                        slot_increment: None,
                        number_of_slots: None,
                        keep_flag: None
                    }),
                    _ => Err("Timeout inconnu.")
                }
            },
            3 => Ok(Self {
                cstype: CSTypes::ITDMA,
                sync_state: u8::from_str_radix(&communication_state[..2], 2).unwrap(),
                slot_timeout: None,
                slot_offset: None,
                utc_hour: None,
                utc_minute: None,
                slot_number: None,
                received_stations: None,
                slot_increment: Some(u16::from_str_radix(&communication_state[2..15], 2).unwrap()),
                number_of_slots: Some(u8::from_str_radix(&communication_state[15..18], 2).unwrap()),
                keep_flag: Some(if u8::from_str_radix(&communication_state[18..19], 2).unwrap() == 1 {true} else {false})
            }),
            _ => Err("Message de type inconnu ou pas encore implémenté.")
        }
    }


    pub fn build_sub_message(&self) -> Result<String, &'static str> {
        if (self.slot_timeout.unwrap() == 3 || self.slot_timeout.unwrap() == 5 || self.slot_timeout.unwrap() == 7) && self.received_stations.is_some() {
            Ok(uint_to_bits(self.received_stations.unwrap(), Some(14)))
        } else if (self.slot_timeout.unwrap() == 2 || self.slot_timeout.unwrap() == 4 || self.slot_timeout.unwrap() == 6) && self.slot_number.is_some() {
            Ok(uint_to_bits(self.slot_number.unwrap(), Some(14)))
        } else if self.slot_timeout.unwrap() == 1 {
            Ok(String::from("000") + &uint_to_bits(get_current_datetime().hour(), Some(5)) + &uint_to_bits(get_current_datetime().minute(), Some(6)))
        } else if self.slot_timeout.unwrap() == 0 && self.slot_offset.is_some() {
            Ok(uint_to_bits(self.slot_offset.unwrap(), Some(14)))
        } else {
            Err("Combinaison d'arguments incohérente.")
        }
    }


    pub fn build(&self) -> Result<String, &'static str> {
        match self.cstype {
            CSTypes::SOTDMA => {
                if self.slot_timeout.is_some() && (self.slot_offset.is_some() || (self.utc_hour.is_some() && self.utc_minute.is_some()) || self.slot_number.is_some() || self.received_stations.is_some())  {
                    Ok(uint_to_bits(self.sync_state, Some(2)) + &uint_to_bits(self.slot_timeout.unwrap(), Some(3)) + &self.build_sub_message().unwrap())
                } else {
                    Err("Combinaison d'attributs incohérente.")
                }
            },
            CSTypes::ITDMA => {
                if self.slot_increment.is_some() && self.number_of_slots.is_some() && self.keep_flag.is_some() {
                    Ok(uint_to_bits(self.sync_state, Some(2)) + &uint_to_bits(self.slot_increment.unwrap(), Some(13)) + &uint_to_bits(self.number_of_slots.unwrap(), Some(3)) + &uint_to_bits(if self.keep_flag.unwrap() {1} else {0}, Some(1)))
                } else {
                    Err("Combinaison d'attributs incohérente.")
                }
            }
        }
    }
}


impl Message {
    pub fn init(msg: Option<String>, boat_info: Option<BoatInfo>, message_type: Option<u8>, communication_state: Option<CommunicationState>) -> Result<Self, &'static str> {
        if msg.is_some() && message_type.is_none() && boat_info.is_none() && communication_state.is_none() {
            let (message_type, data, communication_state, crc, boat_info) = Message::parse(&msg.unwrap()).unwrap();

            Ok(Self {
                message_type: message_type,
                boat_info: boat_info,

                ramp_up_bits: String::from("11111111"),
                sync_sequence: String::from("010101010101010101010101"),
                start_flag: String::from("01111110"),
                data: data,
                communication_state: communication_state,
                crc: crc,
                end_flag: String::from("01111110"),
                buffer: String::from("11111111111111111111111")
            })
        } else if boat_info.is_some() && message_type.is_some() && msg.is_none() {
            let data: String = Message::build_data_string(boat_info.as_ref().unwrap(), message_type.unwrap()).unwrap();

            Ok(Self {
                message_type: message_type.unwrap(),
                boat_info: boat_info.unwrap(),

                ramp_up_bits: String::from("11111111"),
                sync_sequence: String::from("010101010101010101010101"),
                start_flag: String::from("01111110"),
                data: data.clone(),
                communication_state: communication_state.clone(),
                crc: Message::compute_crc_string(Message::build_payload(message_type.unwrap(), &data, communication_state)),
                end_flag: String::from("01111110"),
                buffer: String::from("11111111111111111111111")
            })
        } else {
            Err("Combinaison d'arguments incohérente.")
        }
    }


    pub fn message_type(msg: &str) -> u8 {
        u8::from_str_radix(&msg[40..46], 2).unwrap()
    }


    pub fn compute_crc_string(msg: String) -> String {
        format!("{:016b}", X25.checksum(msg.as_bytes()))
    }


    pub fn build_data_string(boat_info: &BoatInfo, msg_type: u8) -> Result<String, &'static str> {
        match msg_type {
            1 | 2 | 3 => {
                let mut data_vec: Vec<String> = Vec::new();

                let _ =  MSG123_FIELDS.iter()
                .for_each(|field|  {
                    data_vec.push(boat_info.get_as_bits(field, msg_type));
                });
                Ok(data_vec.join(""))
            },
            5 => {
                let mut data_vec: Vec<String> = Vec::new();

                let _ = MSG5_FIELDS.iter()
                .for_each(|field|  {
                    data_vec.push(boat_info.get_as_bits(field, msg_type));
                });

                Ok(data_vec.join(""))
            },
            _ => Err("Type de message inconnu ou pas encore implémenté.")
        }
    }


    pub fn build_payload(msg_type: u8, data: &str, communication_state: Option<CommunicationState>) -> String {
        if communication_state.is_none() {
            String::from(uint_to_bits(msg_type, Some(6)) + "11" + data)
        } else {
            String::from(uint_to_bits(msg_type, Some(6)) + "11" + data + &communication_state.unwrap().build().unwrap())
        }
    }


    pub fn build(&self) -> String {
        let payload: String = Message::build_payload(self.message_type, &Message::build_data_string(&self.boat_info, self.message_type).unwrap(), self.communication_state.clone());
        let msg = self.ramp_up_bits.clone() + &self.sync_sequence.clone() + &self.start_flag + &payload + &Message::compute_crc_string(payload.clone()) + &self.end_flag + &self.buffer;
        msg
    }


    pub fn parse(msg: &str) -> Result<(u8, String, Option<CommunicationState>, String, BoatInfo), &'static str> {
        let msg_type: u8 = Message::message_type(msg);
        match msg_type {
            1 | 2 | 3 => {
                let payload: String = String::from(&msg[40..208]);
                let data: String = String::from(&payload[8..149]);
                let communication_state: CommunicationState = CommunicationState::parse(&payload[149..168], msg_type).unwrap();
                let msg_crc: String = String::from(&msg[208..224]);
                let computed_crc: String = Message::compute_crc_string(payload); // ATTENTION ! A remplacer après le POC car ici, calcule le checksum de la string représentant le binaire et pas le binaire directement

                if msg_crc == computed_crc {
                    let boat_info: BoatInfo = BoatInfo {
                        mmsi: AtomicU32::new(u32::from_str_radix(&data[0..30], 2).unwrap()),
                        imo_number: RwLock::new(None),
                        call_sign: RwLock::new(None),
                        name: RwLock::new(None),
                        type_of_ship_and_cargo_type: RwLock::new(None),
                        position_accuracy: RwLock::new(Some(u8::from_str_radix(&data[52..53], 2).unwrap())),
                        ais_version: RwLock::new(None),
                        type_of_epf_device: RwLock::new(None),
                        a: RwLock::new(None),
                        b: RwLock::new(None),
                        c: RwLock::new(None),
                        d: RwLock::new(None),
                        destination: RwLock::new(None),
                        navigational_status: RwLock::new(Some(u8::from_str_radix(&data[30..34], 2).unwrap())),
                        time_stamp: RwLock::new(Some(u8::from_str_radix(&data[129..135], 2).unwrap())),
                        eta_month: RwLock::new(None),
                        eta_day: RwLock::new(None),
                        eta_hour: RwLock::new(None),
                        eta_minute: RwLock::new(None),
                        maximum_present_static_draught: RwLock::new(None),
                        dte: RwLock::new(None),
                        spare: RwLock::new(Some(u8::from_str_radix(&data[137..140], 2).unwrap())),
                        special_maneuvre_indicator: RwLock::new(Some(u8::from_str_radix(&data[135..137], 2).unwrap())),
                        raim_flag: RwLock::new(Some(u8::from_str_radix(&data[140..141], 2).unwrap())),
                        latitude: RwLock::new(Some(u32::from_str_radix(&data[53..81], 2).unwrap())),
                        longitude: RwLock::new(Some(u32::from_str_radix(&data[81..108], 2).unwrap())),
                        course_over_ground: RwLock::new(Some(u16::from_str_radix(&data[108..120], 2).unwrap())),
                        speed_over_ground: RwLock::new(Some(u16::from_str_radix(&data[42..52], 2).unwrap())),
                        rate_of_turn: RwLock::new(Some(u8::from_str_radix(&data[34..42], 2).unwrap())),
                        true_heading: RwLock::new(Some(u16::from_str_radix(&data[120..129], 2).unwrap()))
                    };

                    Ok((msg_type, data, Some(communication_state), msg_crc, boat_info))
                } else {
                    Err("Payload du message corrompu.")
                }
            },
            5 => {
                let payload: String = String::from(&msg[40..464]);
                let data: String = String::from(&payload[8..]);
                let msg_crc: String = String::from(&msg[464..480]);
                let computed_crc: String = Message::compute_crc_string(payload); // ATTENTION ! A remplacer après le POC car ici, calcule le checksum de la string représentant le binaire et pas le binaire directement
            
                if msg_crc == computed_crc {
                    let boat_info: BoatInfo = BoatInfo {
                        mmsi: AtomicU32::new(u32::from_str_radix(&data[0..30], 2).unwrap()),
                        imo_number:RwLock::new(Some(u32::from_str_radix(&data[32..62], 2).unwrap())),
                        call_sign:RwLock::new(Some(bits_to_string(&data[62..104]))),
                        name:RwLock::new(Some(bits_to_string(&data[104..224]))),
                        type_of_ship_and_cargo_type:RwLock::new(Some(u8::from_str_radix(&data[224..232], 2).unwrap())),
                        position_accuracy:RwLock::new(None),
                        ais_version:RwLock::new(Some(u8::from_str_radix(&data[30..32], 2).unwrap())),
                        type_of_epf_device:RwLock::new(Some(u8::from_str_radix(&data[262..266], 2).unwrap())),
                        a:RwLock::new(Some(u16::from_str_radix(&data[232..241], 2).unwrap())),
                        b:RwLock::new(Some(u16::from_str_radix(&data[241..250], 2).unwrap())),
                        c:RwLock::new(Some(u8::from_str_radix(&data[250..256], 2).unwrap())),
                        d:RwLock::new(Some(u8::from_str_radix(&data[256..262], 2).unwrap())),
                        destination:RwLock::new(Some(bits_to_string(&data[294..414]))),
                        navigational_status:RwLock::new(None),
                        time_stamp:RwLock::new(None),
                        eta_month:RwLock::new(Some(u8::from_str_radix(&data[282..286], 2).unwrap())),
                        eta_day:RwLock::new(Some(u8::from_str_radix(&data[277..282], 2).unwrap())),
                        eta_hour:RwLock::new(Some(u8::from_str_radix(&data[272..277], 2).unwrap())),
                        eta_minute:RwLock::new(Some(u8::from_str_radix(&data[266..272], 2).unwrap())),
                        maximum_present_static_draught:RwLock::new(Some(u8::from_str_radix(&data[286..294], 2).unwrap())),
                        dte:RwLock::new(Some(u8::from_str_radix(&data[414..415], 2).unwrap())),
                        spare:RwLock::new(Some(u8::from_str_radix(&data[415..416], 2).unwrap())),
                        special_maneuvre_indicator:RwLock::new(None),
                        raim_flag:RwLock::new(None),
                        latitude:RwLock::new(None),
                        longitude:RwLock::new(None),
                        course_over_ground:RwLock::new(None),
                        speed_over_ground:RwLock::new(None),
                        rate_of_turn:RwLock::new(None),
                        true_heading:RwLock::new(None)
                    };

                    Ok((msg_type, data, None, msg_crc, boat_info))
                } else {
                    Err("Payload du message corrompu.")
                }
            },
            _ => Err("Message de type inconnu ou pas encore implémenté.")
        }
    }
}