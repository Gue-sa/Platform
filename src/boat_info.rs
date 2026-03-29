use std::sync::{RwLock, atomic::AtomicU32};

use crate::{common::utils::{string_to_bits, uint_to_bits}, impl_atomic_access, impl_rwlock_access};


#[derive(Debug)]
pub struct BoatInfo {
    pub mmsi: AtomicU32,
    pub imo_number: RwLock<Option<u32>>,
    pub call_sign: RwLock<Option<String>>,
    pub name: RwLock<Option<String>>,
    pub type_of_ship_and_cargo_type: RwLock<Option<u8>>,
    pub position_accuracy: RwLock<Option<u8>>,
    pub ais_version: RwLock<Option<u8>>,
    pub type_of_epf_device: RwLock<Option<u8>>,
    pub a: RwLock<Option<u16>>,
    pub b: RwLock<Option<u16>>,
    pub c: RwLock<Option<u8>>,
    pub d: RwLock<Option<u8>>,
    pub destination: RwLock<Option<String>>,
    pub navigational_status: RwLock<Option<u8>>,
    pub time_stamp: RwLock<Option<u8>>,
    pub eta_month: RwLock<Option<u8>>,
    pub eta_day: RwLock<Option<u8>>,
    pub eta_hour: RwLock<Option<u8>>,
    pub eta_minute: RwLock<Option<u8>>,
    pub maximum_present_static_draught: RwLock<Option<u8>>,
    pub dte: RwLock<Option<u8>>,
    pub spare: RwLock<Option<u8>>,
    pub special_maneuvre_indicator: RwLock<Option<u8>>,
    pub raim_flag: RwLock<Option<u8>>,
    pub latitude: RwLock<Option<u32>>,
    pub longitude: RwLock<Option<u32>>,
    pub course_over_ground: RwLock<Option<u16>>,
    pub speed_over_ground: RwLock<Option<u16>>,
    pub rate_of_turn: RwLock<Option<u8>>,
    pub true_heading: RwLock<Option<u16>>
}


impl BoatInfo {
    pub fn init() -> Self {
        Self {
            mmsi: AtomicU32::new(3),
            imo_number: RwLock::new(Some(5)),
            call_sign: RwLock::new(Some(String::from("default"))),
            name: RwLock::new(Some(String::from("default"))),
            type_of_ship_and_cargo_type: RwLock::new(Some(0)),
            position_accuracy: RwLock::new(Some(0)),
            ais_version: RwLock::new(Some(0)),
            type_of_epf_device: RwLock::new(Some(0)),
            a: RwLock::new(Some(0)),
            b: RwLock::new(Some(0)),
            c: RwLock::new(Some(0)),
            d: RwLock::new(Some(0)),
            destination: RwLock::new(Some(String::from("default"))),
            navigational_status: RwLock::new(Some(0)),
            time_stamp: RwLock::new(Some(0)),
            eta_month: RwLock::new(Some(0)),
            eta_day: RwLock::new(Some(0)),
            eta_hour: RwLock::new(Some(0)),
            eta_minute: RwLock::new(Some(0)),
            maximum_present_static_draught: RwLock::new(Some(0)),
            dte: RwLock::new(Some(0)),
            spare: RwLock::new(Some(0)),
            special_maneuvre_indicator: RwLock::new(Some(0)),
            raim_flag: RwLock::new(Some(0)),
            latitude: RwLock::new(Some(0)),
            longitude: RwLock::new(Some(0)),
            course_over_ground: RwLock::new(Some(0)),
            speed_over_ground: RwLock::new(Some(0)),
            rate_of_turn: RwLock::new(Some(0)),
            true_heading: RwLock::new(Some(0))
        }
    }


    impl_atomic_access!(mmsi, u32, mmsi, set_mmsi);

    impl_rwlock_access!(imo_number, Option<u32>, imo_number, set_imo_number);
    impl_rwlock_access!(call_sign, Option<String>, call_sign, set_call_sign);
    impl_rwlock_access!(name, Option<String>, name, set_name);
    impl_rwlock_access!(type_of_ship_and_cargo_type, Option<u8>, type_of_ship_and_cargo_type, set_type_of_ship_and_cargo_type);
    impl_rwlock_access!(ais_version, Option<u8>, ais_version, set_ais_version);
    impl_rwlock_access!(type_of_epf_device, Option<u8>, type_of_epf_device, set_type_of_epf_device);
    impl_rwlock_access!(a, Option<u16>, a, set_a);
    impl_rwlock_access!(b, Option<u16>, b, set_b);
    impl_rwlock_access!(c, Option<u8>, c, set_c);
    impl_rwlock_access!(d, Option<u8>, d, set_d);
    impl_rwlock_access!(destination, Option<String>, destination, set_destination);
    impl_rwlock_access!(eta_month, Option<u8>, eta_month, set_eta_month);
    impl_rwlock_access!(eta_day, Option<u8>, eta_day, set_eta_day);
    impl_rwlock_access!(eta_hour, Option<u8>, eta_hour, set_eta_hour);
    impl_rwlock_access!(eta_minute, Option<u8>, eta_minute, set_eta_minute);
    impl_rwlock_access!(maximum_present_static_draught, Option<u8>, maximum_present_static_draught, set_maximum_present_static_draught);
    impl_rwlock_access!(navigational_status, Option<u8>, navigational_status, set_navigational_status);
    impl_rwlock_access!(latitude, Option<u32>, latitude, set_latitude);
    impl_rwlock_access!(longitude, Option<u32>, longitude, set_longitude);
    impl_rwlock_access!(course_over_ground, Option<u16>, course_over_ground, set_course_over_ground);
    impl_rwlock_access!(speed_over_ground, Option<u16>, speed_over_ground, set_speed_over_ground);
    impl_rwlock_access!(true_heading, Option<u16>, true_heading, set_true_heading);
    impl_rwlock_access!(rate_of_turn, Option<u8>, rate_of_turn, set_rate_of_turn);
    impl_rwlock_access!(time_stamp, Option<u8>, time_stamp, set_time_stamp);
    impl_rwlock_access!(position_accuracy, Option<u8>, position_accuracy, set_position_accuracy);
    impl_rwlock_access!(raim_flag, Option<u8>, raim_flag, set_raim_flag);
    impl_rwlock_access!(special_maneuvre_indicator, Option<u8>, special_maneuvre_indicator, set_special_maneuvre_indicator);
    impl_rwlock_access!(dte, Option<u8>, dte, set_dte);
    impl_rwlock_access!(spare, Option<u8>, spare, set_spare);


    pub fn get_as_bits(&self, field_name: &str, msg_type: u8) -> String {
        match field_name {
            "mmsi" => uint_to_bits(self.mmsi(), Some(30)),
            "navigational_status" => uint_to_bits(self.navigational_status().unwrap(), Some(4)),
            "rate_of_turn" => uint_to_bits(self.rate_of_turn().unwrap(), Some(8)),
            "speed_over_ground" => uint_to_bits(self.speed_over_ground().unwrap(), Some(10)),
            "position_accuracy" => uint_to_bits(self.position_accuracy().unwrap(), Some(1)),
            "longitude" => uint_to_bits(self.longitude().unwrap(), Some(28)),
            "latitude" => uint_to_bits(self.latitude().unwrap(), Some(27)),
            "course_over_ground" => uint_to_bits(self.course_over_ground().unwrap(), Some(12)),
            "true_heading" => uint_to_bits(self.true_heading().unwrap(), Some(9)),
            "time_stamp" => uint_to_bits(self.time_stamp().unwrap(), Some(6)),
            "special_maneuvre_indicator" => uint_to_bits(self.special_maneuvre_indicator().unwrap(), Some(2)),
            "raim_flag" => uint_to_bits(self.raim_flag().unwrap(), Some(1)),
            "ais_version" => uint_to_bits(self.ais_version().unwrap(), Some(2)),
            "imo_number" => uint_to_bits(self.imo_number().unwrap(), Some(30)),
            "type_of_ship_and_cargo_type" => uint_to_bits(self.type_of_ship_and_cargo_type().unwrap(), Some(8)),
            "a" => uint_to_bits(self.a().unwrap(), Some(9)),
            "b" => uint_to_bits(self.b().unwrap(), Some(9)),
            "c" => uint_to_bits(self.c().unwrap(), Some(6)),
            "d" => uint_to_bits(self.d().unwrap(), Some(6)),
            "type_of_epf_device" => uint_to_bits(self.type_of_epf_device().unwrap(), Some(4)),
            "eta_month" => uint_to_bits(self.eta_month().unwrap(), Some(4)),
            "eta_day" => uint_to_bits(self.eta_day().unwrap(), Some(5)),
            "eta_hour" => uint_to_bits(self.eta_hour().unwrap(), Some(5)),
            "eta_minute" => uint_to_bits(self.eta_minute().unwrap(), Some(6)),
            "maximum_present_static_draught" => uint_to_bits(self.maximum_present_static_draught().unwrap(), Some(8)),
            "dte" => uint_to_bits(self.dte().unwrap(), Some(1)),
            "spare" => uint_to_bits(self.spare().unwrap_or(0), Some(if msg_type == 5 {1} else {3})),
            "call_sign" => string_to_bits(self.call_sign().as_ref().unwrap().as_ref(), Some(42)),
            "name" => string_to_bits(self.name().as_ref().unwrap().as_ref(), Some(120)),
            "destination" => string_to_bits(self.destination().as_ref().unwrap().as_ref(), Some(120)),
            _ => String::new(),
        }
    }
}


impl Clone for BoatInfo {
    fn clone(&self) -> Self {
        Self {
            mmsi: AtomicU32::new(self.mmsi()),
            imo_number: RwLock::new(Some(self.imo_number().unwrap())),
            call_sign: RwLock::new(Some(self.call_sign().clone().unwrap())),
            name: RwLock::new(Some(self.name().clone().unwrap())),
            type_of_ship_and_cargo_type: RwLock::new(Some(self.type_of_ship_and_cargo_type().unwrap())),
            position_accuracy: RwLock::new(Some(self.position_accuracy().unwrap())),
            ais_version: RwLock::new(Some(self.ais_version().unwrap())),
            type_of_epf_device: RwLock::new(Some(self.type_of_epf_device().unwrap())),
            a: RwLock::new(Some(self.a().unwrap())),
            b: RwLock::new(Some(self.b().unwrap())),
            c: RwLock::new(Some(self.c().unwrap())),
            d: RwLock::new(Some(self.d().unwrap())),
            destination: RwLock::new(Some(self.destination().clone().unwrap())),
            navigational_status: RwLock::new(Some(self.navigational_status().unwrap())),
            time_stamp: RwLock::new(Some(self.time_stamp().unwrap())),
            eta_month: RwLock::new(Some(self.eta_month().unwrap())),
            eta_day: RwLock::new(Some(self.eta_day().unwrap())),
            eta_hour: RwLock::new(Some(self.eta_hour().unwrap())),
            eta_minute: RwLock::new(Some(self.eta_minute().unwrap())),
            maximum_present_static_draught: RwLock::new(Some(self.maximum_present_static_draught().unwrap())),
            dte: RwLock::new(Some(self.dte().unwrap())),
            spare: RwLock::new(Some(self.spare().unwrap())),
            special_maneuvre_indicator: RwLock::new(Some(self.special_maneuvre_indicator().unwrap())),
            raim_flag: RwLock::new(Some(self.raim_flag().unwrap())),
            latitude: RwLock::new(Some(self.latitude().unwrap())),
            longitude: RwLock::new(Some(self.longitude().unwrap())),
            course_over_ground: RwLock::new(Some(self.course_over_ground().unwrap())),
            speed_over_ground: RwLock::new(Some(self.speed_over_ground().unwrap())),
            rate_of_turn: RwLock::new(Some(self.rate_of_turn().unwrap())),
            true_heading: RwLock::new(Some(self.true_heading().unwrap())),
        }
    }
}