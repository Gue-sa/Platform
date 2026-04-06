use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, atomic::AtomicU32};

use crate::{common::{bitpacker::BitPacker, utils::{string_to_bits, uint_to_bits}}, impl_atomic_access, impl_rwlock_access};


#[derive(Debug, Clone)]
pub struct StaticData {
    pub mmsi: u32,
    pub imo_number: u32,
    pub call_sign: String,
    pub name: String,
    pub type_of_ship_and_cargo_type: u8,
    pub position_accuracy: u8,
    pub ais_version: u8,
    pub type_of_epf_device: u8,
    pub a: u16,
    pub b: u16,
    pub c: u8,
    pub d: u8,
    pub spare: u8
}


#[derive(Debug, Clone)]
pub struct VoyageData {
    pub destination: String,
    pub eta_month: u8,
    pub eta_day: u8,
    pub eta_hour: u8,
    pub eta_minute: u8,
    pub maximum_present_static_draught: u8,
    pub dte: u8,
    pub raim_flag: u8
}


#[derive(Debug, Clone)]
pub struct NavigationData {
    pub navigational_status: u8,
    pub time_stamp: u8,
    pub special_maneuvre_indicator: u8,
    pub latitude: u32,
    pub longitude: u32,
    pub course_over_ground: u16,
    pub speed_over_ground: u16,
    pub rate_of_turn: i8,
    pub true_heading: u16
}


#[derive(Debug)]
pub struct BoatInfo {
    static_data: RwLock<StaticData>,
    voyage_data: RwLock<VoyageData>,
    navigation_data: RwLock<NavigationData>
}


impl StaticData {
    pub fn init(mmsi: Option<u32>, imo_number: Option<u32>, call_sign: Option<String>, name: Option<String>, type_of_ship_and_cargo_type: Option<u8>, position_accuracy: Option<u8>,
                ais_version: Option<u8>, type_of_epf_device: Option<u8>, a: Option<u16>, b: Option<u16>, c: Option<u8>, d: Option<u8>) -> Self {
        Self {
            mmsi: mmsi.unwrap_or(0),
            imo_number: imo_number.unwrap_or(0),
            call_sign: call_sign.unwrap_or("default".to_string()),
            name: name.unwrap_or("default".to_string()),
            type_of_ship_and_cargo_type: type_of_ship_and_cargo_type.unwrap_or(0),
            position_accuracy: position_accuracy.unwrap_or(0),
            ais_version: ais_version.unwrap_or(0),
            type_of_epf_device: type_of_epf_device.unwrap_or(0),
            a: a.unwrap_or(0),
            b: b.unwrap_or(0),
            c: c.unwrap_or(0),
            d: d.unwrap_or(0),
            spare: 0
        }
    }
}


impl VoyageData {
    pub fn init(destination: Option<String>, eta_month: Option<u8>, eta_day: Option<u8>, eta_hour: Option<u8>, eta_minute: Option<u8>, maximum_present_static_draught: Option<u8>,
                dte: Option<u8>, raim_flag: Option<u8>) -> Self {
        Self {
            destination: destination.unwrap_or("default".to_string()),
            eta_month: eta_month.unwrap_or(0),
            eta_day: eta_day.unwrap_or(0),
            eta_hour: eta_hour.unwrap_or(24),
            eta_minute: eta_minute.unwrap_or(60),
            maximum_present_static_draught: maximum_present_static_draught.unwrap_or(0),
            dte: dte.unwrap_or(1),
            raim_flag: raim_flag.unwrap_or(0)
        }
    }
}


impl NavigationData {
    pub fn init(navigational_status: Option<u8>, time_stamp: Option<u8>, special_maneuvre_indicator: Option<u8>, latitude: Option<u32>, longitude: Option<u32>,
                course_over_ground: Option<u16>, speed_over_ground: Option<u16>, rate_of_turn: Option<i8>, true_heading: Option<u16>) -> Self {
        Self {
            navigational_status: navigational_status.unwrap_or(15),
            time_stamp: time_stamp.unwrap_or(63),
            special_maneuvre_indicator: special_maneuvre_indicator.unwrap_or(0),
            latitude: latitude.unwrap_or(91),
            longitude: longitude.unwrap_or(181),
            course_over_ground: course_over_ground.unwrap_or(3601),
            speed_over_ground: speed_over_ground.unwrap_or(1023),
            rate_of_turn: rate_of_turn.unwrap_or(-128),
            true_heading: true_heading.unwrap_or(511)
        }
    }
}


impl BoatInfo {
    pub fn init(static_data: Option<StaticData>, voyage_data: Option<VoyageData>, navigation_data: Option<NavigationData>) -> Self {
        Self {
            static_data: RwLock::new(static_data.unwrap_or(StaticData::init(None, None, None, None, None, None, None, None, None, None, None, None))),
            voyage_data: RwLock::new(voyage_data.unwrap_or(VoyageData::init(None, None, None, None, None, None, None, None))),
            navigation_data: RwLock::new(navigation_data.unwrap_or(NavigationData::init(None, None, None, None, None, None, None, None, None)))
        }
    }


    pub fn get_static_data(&self) -> StaticData {
        self.static_data.read().unwrap().clone()
    }


    pub fn get_voyage_data(&self) -> VoyageData {
        self.voyage_data.read().unwrap().clone()
    }


    pub fn get_navigation_data(&self) -> NavigationData {
        self.navigation_data.read().unwrap().clone()
    }


    pub fn update_status(&self, navigational_status: Option<u8>, time_stamp: Option<u8>, special_maneuvre_indicator: Option<u8>) -> () {
        let mut guard: RwLockWriteGuard<'_, NavigationData> = self.navigation_data.write().unwrap();
        guard.navigational_status = navigational_status.unwrap_or(15);
        guard.time_stamp = time_stamp.unwrap_or(63);
        guard.special_maneuvre_indicator = special_maneuvre_indicator.unwrap_or(0);
    }


    pub fn update_positon(&self, latitude: Option<u32>, longitude: Option<u32>) -> () {
        let mut guard: RwLockWriteGuard<'_, NavigationData> = self.navigation_data.write().unwrap();
        guard.latitude = latitude.unwrap_or(91);
        guard.longitude = longitude.unwrap_or(181);
    }


    pub fn update_movement(&self, course_over_ground: Option<u16>, speed_over_ground: Option<u16>, rate_of_turn: Option<i8>, true_heading: Option<u16>) -> () {
        let mut guard: RwLockWriteGuard<'_, NavigationData> = self.navigation_data.write().unwrap();
        guard.course_over_ground = course_over_ground.unwrap_or(3601);
        guard.speed_over_ground = speed_over_ground.unwrap_or(1023);
        guard.rate_of_turn = rate_of_turn.unwrap_or(-128);
        guard.true_heading = true_heading.unwrap_or(511);
    }


    pub fn get_as_bits(&self, field_name: &str, msg_type: u8) -> Result<BitPacker, &'static str> {
        match field_name {
            "mmsi" => Ok(BitPacker::from_int::<u32>(self.get_static_data().mmsi, Some(30))?),
            "navigational_status" => Ok(BitPacker::from_int::<u8>(self.get_navigation_data().navigational_status, Some(4))?),
            "rate_of_turn" => Ok(BitPacker::from_int::<i8>(self.get_navigation_data().rate_of_turn, Some(8))?),
            "speed_over_ground" => Ok(BitPacker::from_int::<u16>(self.get_navigation_data().speed_over_ground, Some(10))?),
            "position_accuracy" => Ok(BitPacker::from_int::<u8>(self.get_static_data().position_accuracy, Some(1))?),
            "longitude" => Ok(BitPacker::from_int::<u32>(self.get_navigation_data().longitude, Some(28))?),
            "latitude" => Ok(BitPacker::from_int::<u32>(self.get_navigation_data().latitude, Some(27))?),
            "course_over_ground" => Ok(BitPacker::from_int::<u16>(self.get_navigation_data().course_over_ground, Some(12))?),
            "true_heading" => Ok(BitPacker::from_int::<u16>(self.get_navigation_data().true_heading, Some(9))?),
            "time_stamp" => Ok(BitPacker::from_int::<u8>(self.get_navigation_data().time_stamp, Some(6))?),
            "special_maneuvre_indicator" => Ok(BitPacker::from_int::<u8>(self.get_navigation_data().special_maneuvre_indicator, Some(2))?),
            "raim_flag" => Ok(BitPacker::from_int::<u8>(self.get_voyage_data().raim_flag, Some(1))?),
            "ais_version" => Ok(BitPacker::from_int::<u8>(self.get_static_data().ais_version, Some(2))?),
            "imo_number" => Ok(BitPacker::from_int::<u32>(self.get_static_data().imo_number, Some(30))?),
            "type_of_ship_and_cargo_type" => Ok(BitPacker::from_int::<u8>(self.get_static_data().type_of_ship_and_cargo_type, Some(8))?),
            "b" => Ok(BitPacker::from_int::<u16>(self.get_static_data().b, Some(9))?),
            "a" => Ok(BitPacker::from_int::<u16>(self.get_static_data().a, Some(9))?),
            "c" => Ok(BitPacker::from_int::<u8>(self.get_static_data().c, Some(6))?),
            "d" => Ok(BitPacker::from_int::<u8>(self.get_static_data().d, Some(6))?),
            "type_of_epf_device" => Ok(BitPacker::from_int::<u8>(self.get_static_data().type_of_epf_device, Some(4))?),
            "eta_month" => Ok(BitPacker::from_int::<u8>(self.get_voyage_data().eta_month, Some(4))?),
            "eta_day" => Ok(BitPacker::from_int::<u8>(self.get_voyage_data().eta_day, Some(5))?),
            "eta_hour" => Ok(BitPacker::from_int::<u8>(self.get_voyage_data().eta_hour, Some(5))?),
            "eta_minute" => Ok(BitPacker::from_int::<u8>(self.get_voyage_data().eta_minute, Some(6))?),
            "maximum_present_static_draught" => Ok(BitPacker::from_int::<u8>(self.get_voyage_data().maximum_present_static_draught, Some(8))?),
            "dte" => Ok(BitPacker::from_int::<u8>(self.get_voyage_data().dte, Some(1))?),
            "spare" => Ok(BitPacker::from_int::<u8>(self.get_static_data().spare, Some(if msg_type == 5 {1} else {3}))?),
            "call_sign" => Ok(BitPacker::from_str(&self.get_static_data().call_sign, Some(42))?),
            "name" => Ok(BitPacker::from_str(&self.get_static_data().name, Some(120))?),
            "destination" => Ok(BitPacker::from_str(&self.get_voyage_data().destination, Some(120))?),
            _ => Err("Champ inconnu.")
        }
    }
}


impl Clone for BoatInfo {
    fn clone(&self) -> Self {
        Self {
            static_data: RwLock::new(self.get_static_data()),
            voyage_data: RwLock::new(self.get_voyage_data()),
            navigation_data: RwLock::new(self.get_navigation_data())
        }
    }
}