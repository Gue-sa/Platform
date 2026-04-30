use crate::{
    bitpacker::BitPacker,
    common::errors::{BoatInfoError, BoatInfoResult},
};
use getset::{Getters, Setters};
use serde::Serialize;
use std::sync::{RwLock, RwLockWriteGuard};

#[derive(Debug, Clone, Serialize, Getters, Setters, PartialEq)]
#[getset(get = "pub", set = "pub")]
pub struct StaticData {
    mmsi: u32,
    imo_number: u32,
    call_sign: String,
    name: String,
    type_of_ship_and_cargo_type: u8,
    position_accuracy: u8,
    ais_version: u8,
    type_of_epf_device: u8,
    a: u16,
    b: u16,
    c: u8,
    d: u8,
    spare: u8,
}

#[derive(Debug, Clone, Serialize, Getters, Setters, PartialEq)]
#[getset(get = "pub", set = "pub")]
pub struct VoyageData {
    destination: String,
    eta_month: u8,
    eta_day: u8,
    eta_hour: u8,
    eta_minute: u8,
    maximum_present_static_draught: u8,
    dte: u8,
    raim_flag: u8,
}

#[derive(Debug, Clone, Serialize, Getters, Setters, PartialEq)]
#[getset(get = "pub", set = "pub")]
pub struct NavigationData {
    navigational_status: u8,
    time_stamp: u8,
    special_maneuvre_indicator: u8,
    latitude: u32,
    longitude: u32,
    course_over_ground: u16,
    speed_over_ground: u16,
    rate_of_turn: i8,
    true_heading: u16,
}

#[derive(Debug, Serialize)]
pub struct BoatInfo {
    static_data: RwLock<StaticData>,
    voyage_data: RwLock<VoyageData>,
    navigation_data: RwLock<NavigationData>,
}

impl StaticData {
    pub fn new(
        mmsi: Option<u32>,
        imo_nbr: Option<u32>,
        call_sign: Option<String>,
        name: Option<String>,
        type_of_ship_and_cargo_type: Option<u8>,
        pos_accuracy: Option<u8>,
        ais_version: Option<u8>,
        type_of_epf_device: Option<u8>,
        a: Option<u16>,
        b: Option<u16>,
        c: Option<u8>,
        d: Option<u8>,
    ) -> Self {
        Self {
            mmsi: mmsi.unwrap_or(0),
            imo_number: imo_nbr.unwrap_or(0),
            call_sign: call_sign.unwrap_or("@@@@@@@".to_string()),
            name: name.unwrap_or("@@@@@@@@@@@@@@@@@@@@".to_string()),
            type_of_ship_and_cargo_type: type_of_ship_and_cargo_type.unwrap_or(0),
            position_accuracy: pos_accuracy.unwrap_or(0),
            ais_version: ais_version.unwrap_or(0),
            type_of_epf_device: type_of_epf_device.unwrap_or(0),
            a: a.unwrap_or(0),
            b: b.unwrap_or(0),
            c: c.unwrap_or(0),
            d: d.unwrap_or(0),
            spare: 0,
        }
    }
}

impl VoyageData {
    pub fn new(
        dest: Option<String>,
        eta_month: Option<u8>,
        eta_day: Option<u8>,
        eta_hour: Option<u8>,
        eta_min: Option<u8>,
        maximum_present_static_draught: Option<u8>,
        dte: Option<u8>,
        raim_flag: Option<u8>,
    ) -> Self {
        Self {
            destination: dest.unwrap_or("@@@@@@@@@@@@@@@@@@@@".to_string()),
            eta_month: eta_month.unwrap_or(0),
            eta_day: eta_day.unwrap_or(0),
            eta_hour: eta_hour.unwrap_or(24),
            eta_minute: eta_min.unwrap_or(60),
            maximum_present_static_draught: maximum_present_static_draught.unwrap_or(0),
            dte: dte.unwrap_or(1),
            raim_flag: raim_flag.unwrap_or(0),
        }
    }
}

impl NavigationData {
    pub fn new(
        nav_status: Option<u8>,
        time_stamp: Option<u8>,
        special_maneuvre_indicator: Option<u8>,
        lat: Option<u32>,
        lon: Option<u32>,
        course_over_ground: Option<u16>,
        speed_over_ground: Option<u16>,
        rate_of_turn: Option<i8>,
        true_heading: Option<u16>,
    ) -> Self {
        Self {
            navigational_status: nav_status.unwrap_or(15),
            time_stamp: time_stamp.unwrap_or(63),
            special_maneuvre_indicator: special_maneuvre_indicator.unwrap_or(0),
            latitude: lat.unwrap_or(0),
            longitude: lon.unwrap_or(0),
            course_over_ground: course_over_ground.unwrap_or(3601),
            speed_over_ground: speed_over_ground.unwrap_or(1023),
            rate_of_turn: rate_of_turn.unwrap_or(-128),
            true_heading: true_heading.unwrap_or(511),
        }
    }
}

impl BoatInfo {
    pub fn new(
        static_data: Option<StaticData>,
        voyage_data: Option<VoyageData>,
        navigation_data: Option<NavigationData>,
    ) -> Self {
        Self {
            static_data: RwLock::new(static_data.unwrap_or(StaticData::new(
                None, None, None, None, None, None, None, None, None, None, None, None,
            ))),
            voyage_data: RwLock::new(voyage_data.unwrap_or(VoyageData::new(
                None, None, None, None, None, None, None, None,
            ))),
            navigation_data: RwLock::new(navigation_data.unwrap_or(NavigationData::new(
                None, None, None, None, None, None, None, None, None,
            ))),
        }
    }

    pub fn get_static_data(&self) -> BoatInfoResult<StaticData> {
        Ok(self
            .static_data
            .read()
            .map_err(|_| BoatInfoError::StaticDataPoisoned)?
            .clone())
    }

    pub fn get_voyage_data(&self) -> BoatInfoResult<VoyageData> {
        Ok(self
            .voyage_data
            .read()
            .map_err(|_| BoatInfoError::VoyageDataPoisoned)?
            .clone())
    }

    pub fn get_writeable_voyage_data(&self) -> BoatInfoResult<RwLockWriteGuard<'_, VoyageData>> {
        Ok(self
            .voyage_data
            .write()
            .map_err(|_| BoatInfoError::VoyageDataPoisoned)?)
    }

    pub fn get_navigation_data(&self) -> BoatInfoResult<NavigationData> {
        Ok(self
            .navigation_data
            .read()
            .map_err(|_| BoatInfoError::NavigationDataPoisoned)?
            .clone())
    }

    pub fn get_writeable_navigation_data(
        &self,
    ) -> BoatInfoResult<RwLockWriteGuard<'_, NavigationData>> {
        Ok(self
            .navigation_data
            .write()
            .map_err(|_| BoatInfoError::NavigationDataPoisoned)?)
    }

    pub fn update_status(
        &self,
        nav_status: Option<u8>,
        time_stamp: Option<u8>,
        special_maneuvre_indicator: Option<u8>,
    ) -> BoatInfoResult<()> {
        let mut guard: RwLockWriteGuard<'_, NavigationData> =
            self.get_writeable_navigation_data()?;
        guard.navigational_status = nav_status.unwrap_or(15);
        guard.time_stamp = time_stamp.unwrap_or(63);
        guard.special_maneuvre_indicator = special_maneuvre_indicator.unwrap_or(0);

        Ok(())
    }

    pub fn update_positon(&self, lat: Option<u32>, lon: Option<u32>) -> BoatInfoResult<()> {
        let mut guard: RwLockWriteGuard<'_, NavigationData> =
            self.get_writeable_navigation_data()?;
        guard.latitude = lat.unwrap_or(0);
        guard.longitude = lon.unwrap_or(0);

        Ok(())
    }

    pub fn update_movement(
        &self,
        course_over_ground: Option<u16>,
        speed_over_ground: Option<u16>,
        rate_of_turn: Option<i8>,
        true_heading: Option<u16>,
    ) -> BoatInfoResult<()> {
        let mut guard: RwLockWriteGuard<'_, NavigationData> =
            self.get_writeable_navigation_data()?;

        guard.course_over_ground = course_over_ground.unwrap_or(3601);
        guard.speed_over_ground = speed_over_ground.unwrap_or(1023);
        guard.rate_of_turn = rate_of_turn.unwrap_or(-128);
        guard.true_heading = true_heading.unwrap_or(511);

        Ok(())
    }

    pub fn update_voyage_data(
        &self,
        dest: Option<String>,
        eta_month: Option<u8>,
        eta_day: Option<u8>,
        eta_hour: Option<u8>,
        eta_min: Option<u8>,
    ) -> BoatInfoResult<()> {
        let mut guard: RwLockWriteGuard<'_, VoyageData> = self.get_writeable_voyage_data()?;

        guard.destination = dest.unwrap_or("@@@@@@@@@@@@@@@@@@@@".to_string());
        guard.eta_month = eta_month.unwrap_or(0);
        guard.eta_day = eta_day.unwrap_or(0);
        guard.eta_hour = eta_hour.unwrap_or(24);
        guard.eta_minute = eta_min.unwrap_or(60);

        Ok(())
    }

    pub fn to_bits(&self, field: &str, msg_type: u8) -> BoatInfoResult<BitPacker> {
        match field {
            "mmsi" => Ok(BitPacker::from_int::<u32>(
                self.get_static_data()?.mmsi,
                Some(30),
            )),
            "navigational_status" => Ok(BitPacker::from_int::<u8>(
                self.get_navigation_data()?.navigational_status,
                Some(4),
            )),
            "rate_of_turn" => Ok(BitPacker::from_int::<i8>(
                self.get_navigation_data()?.rate_of_turn,
                Some(8),
            )),
            "speed_over_ground" => Ok(BitPacker::from_int::<u16>(
                self.get_navigation_data()?.speed_over_ground,
                Some(10),
            )),
            "position_accuracy" => Ok(BitPacker::from_int::<u8>(
                self.get_static_data()?.position_accuracy,
                Some(1),
            )),
            "longitude" => Ok(BitPacker::from_int::<u32>(
                self.get_navigation_data()?.longitude,
                Some(28),
            )),
            "latitude" => Ok(BitPacker::from_int::<u32>(
                self.get_navigation_data()?.latitude,
                Some(27),
            )),
            "course_over_ground" => Ok(BitPacker::from_int::<u16>(
                self.get_navigation_data()?.course_over_ground,
                Some(12),
            )),
            "true_heading" => Ok(BitPacker::from_int::<u16>(
                self.get_navigation_data()?.true_heading,
                Some(9),
            )),
            "time_stamp" => Ok(BitPacker::from_int::<u8>(
                self.get_navigation_data()?.time_stamp,
                Some(6),
            )),
            "special_maneuvre_indicator" => Ok(BitPacker::from_int::<u8>(
                self.get_navigation_data()?.special_maneuvre_indicator,
                Some(2),
            )),
            "raim_flag" => Ok(BitPacker::from_int::<u8>(
                self.get_voyage_data()?.raim_flag,
                Some(1),
            )),
            "ais_version" => Ok(BitPacker::from_int::<u8>(
                self.get_static_data()?.ais_version,
                Some(2),
            )),
            "imo_number" => Ok(BitPacker::from_int::<u32>(
                self.get_static_data()?.imo_number,
                Some(30),
            )),
            "type_of_ship_and_cargo_type" => Ok(BitPacker::from_int::<u8>(
                self.get_static_data()?.type_of_ship_and_cargo_type,
                Some(8),
            )),
            "b" => Ok(BitPacker::from_int::<u16>(
                self.get_static_data()?.b,
                Some(9),
            )),
            "a" => Ok(BitPacker::from_int::<u16>(
                self.get_static_data()?.a,
                Some(9),
            )),
            "c" => Ok(BitPacker::from_int::<u8>(
                self.get_static_data()?.c,
                Some(6),
            )),
            "d" => Ok(BitPacker::from_int::<u8>(
                self.get_static_data()?.d,
                Some(6),
            )),
            "type_of_epf_device" => Ok(BitPacker::from_int::<u8>(
                self.get_static_data()?.type_of_epf_device,
                Some(4),
            )),
            "eta_month" => Ok(BitPacker::from_int::<u8>(
                self.get_voyage_data()?.eta_month,
                Some(4),
            )),
            "eta_day" => Ok(BitPacker::from_int::<u8>(
                self.get_voyage_data()?.eta_day,
                Some(5),
            )),
            "eta_hour" => Ok(BitPacker::from_int::<u8>(
                self.get_voyage_data()?.eta_hour,
                Some(5),
            )),
            "eta_minute" => Ok(BitPacker::from_int::<u8>(
                self.get_voyage_data()?.eta_minute,
                Some(6),
            )),
            "maximum_present_static_draught" => Ok(BitPacker::from_int::<u8>(
                self.get_voyage_data()?.maximum_present_static_draught,
                Some(8),
            )),
            "dte" => Ok(BitPacker::from_int::<u8>(
                self.get_voyage_data()?.dte,
                Some(1),
            )),
            "spare" => Ok(BitPacker::from_int::<u8>(
                self.get_static_data()?.spare,
                Some(if msg_type == 5 { 1 } else { 3 }),
            )),
            "call_sign" => Ok(BitPacker::from_str(
                &self.get_static_data()?.call_sign,
                Some(42),
            )),
            "name" => Ok(BitPacker::from_str(
                &self.get_static_data()?.name,
                Some(120),
            )),
            "destination" => Ok(BitPacker::from_str(
                &self.get_voyage_data()?.destination,
                Some(120),
            )),
            _ => Ok(BitPacker::from_int(0, None)),
        }
    }
}

impl Clone for BoatInfo {
    fn clone(&self) -> Self {
        Self {
            static_data: RwLock::new(self.get_static_data().unwrap_or(StaticData::new(
                None, None, None, None, None, None, None, None, None, None, None, None,
            ))),
            voyage_data: RwLock::new(self.get_voyage_data().unwrap_or(VoyageData::new(
                None, None, None, None, None, None, None, None,
            ))),
            navigation_data: RwLock::new(self.get_navigation_data().unwrap_or(
                NavigationData::new(None, None, None, None, None, None, None, None, None),
            )),
        }
    }
}
