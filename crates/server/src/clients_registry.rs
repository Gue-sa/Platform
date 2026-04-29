use std::{collections::HashSet, net::IpAddr, sync::RwLock};

use shared::common::types::{ClientsRegistryError, ClientsRegistryResult};

pub struct ClientsRegistry {
    clients: RwLock<HashSet<IpAddr>>,
}

impl ClientsRegistry {
    pub fn init() -> Self {
        Self {
            clients: RwLock::new(HashSet::new()),
        }
    }

    pub fn is_registered(&self, client: &IpAddr) -> ClientsRegistryResult<bool> {
        Ok(self
            .clients
            .read()
            .map_err(|_| ClientsRegistryError::ClientsRegistryPoisoned)?
            .contains(client))
    }

    pub fn register_client(&self, client: IpAddr) -> ClientsRegistryResult<()> {
        self.clients
            .write()
            .map_err(|_| ClientsRegistryError::ClientsRegistryPoisoned)?
            .insert(client);

        Ok(())
    }

    pub fn unregister_client(&self, client: IpAddr) -> ClientsRegistryResult<()> {
        self.clients
            .write()
            .map_err(|_| ClientsRegistryError::ClientsRegistryPoisoned)?
            .remove(&client);

        Ok(())
    }

    pub fn get(&self) -> ClientsRegistryResult<Box<[IpAddr]>> {
        Ok(self
            .clients
            .read()
            .map_err(|_| ClientsRegistryError::ClientsRegistryPoisoned)?
            .iter()
            .cloned()
            .collect::<Vec<IpAddr>>()
            .into_boxed_slice())
    }
}
