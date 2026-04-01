use dashmap::{DashMap, mapref::one::Ref};

use crate::boat_info::BoatInfo;


pub struct BoatsInfoRegistry {
    registry: DashMap<u32, BoatInfo>
}


impl BoatsInfoRegistry {
    pub fn init() -> Self {
        Self {
            registry: DashMap::new()
        }
    }


    pub fn register(&mut self, info: BoatInfo) -> () {
        self.registry.insert(info.get_static_data().mmsi, info);
    }


    pub fn is_registered(&self, mmsi: &u32) -> bool {
        self.registry.contains_key(mmsi)
    }


    pub fn get(&self, mmsi: u32) -> Option<BoatInfo> {
        self.registry.get(&mmsi).map(|boat_ref: Ref<'_, u32, BoatInfo>| boat_ref.value().clone())
    }


    pub fn update(&mut self, new_boat_info: BoatInfo) -> () {
        let mmsi: u32 = new_boat_info.get_static_data().mmsi;
        if self.is_registered(&mmsi) {
            self.registry.insert(mmsi, new_boat_info);
        }
    }


    pub fn unregister(&mut self, mmsi: u32) -> Option<BoatInfo> {
        self.registry.remove(&mmsi).map(|(_, boat)| boat)
    }
}