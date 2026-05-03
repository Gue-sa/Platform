use crate::{
    boat_info::BoatInfo,
    common::errors::{BoatsRegistryError, BoatsRegistryResult},
};
use dashmap::{DashMap, mapref::one::Ref};

pub struct BoatsInfoRegistry {
    registry: DashMap<u32, BoatInfo>,
}

impl BoatsInfoRegistry {
    pub fn new() -> Self {
        Self {
            registry: DashMap::new(),
        }
    }

    pub fn register(&self, info: BoatInfo) -> BoatsRegistryResult<()> {
        if self.is_registered(info.get_static_data()?.mmsi()) {
            Err(BoatsRegistryError::MmsiAlreadyRegistered)
        } else {
            self.registry.insert(*info.get_static_data()?.mmsi(), info);

            Ok(())
        }
    }

    pub fn is_registered(&self, mmsi: &u32) -> bool {
        self.registry.contains_key(mmsi)
    }

    pub fn get(&self, mmsi: u32) -> BoatsRegistryResult<BoatInfo> {
        self.registry
            .get(&mmsi)
            .map(|boat_ref: Ref<'_, u32, BoatInfo>| boat_ref.value().clone())
            .ok_or(BoatsRegistryError::UnkownMmsi)
    }

    pub fn update(&self, new_boat_info: BoatInfo) -> BoatsRegistryResult<BoatInfo> {
        let mmsi = *new_boat_info.get_static_data()?.mmsi();
        if self.is_registered(&mmsi) {
            Ok(self.registry.insert(mmsi, new_boat_info).unwrap())
        } else {
            Err(BoatsRegistryError::MmsiAlreadyRegistered)
        }
    }

    pub fn unregister(&mut self, mmsi: u32) -> BoatsRegistryResult<BoatInfo> {
        if self.is_registered(&mmsi) {
            Ok(self.registry.remove(&mmsi).map(|(_, boat)| boat).unwrap())
        } else {
            Err(BoatsRegistryError::UnkownMmsi)
        }
    }

    pub fn length(&self) -> usize {
        self.registry.len()
    }

    pub fn export(&self) -> Box<[(u32, BoatInfo)]> {
        self.registry
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect()
    }
}
