use std::{collections::HashSet, net::IpAddr, sync::RwLock};

pub struct ClientsRegistry {
    clients: RwLock<HashSet<IpAddr>>,
}

impl ClientsRegistry {
    pub fn init() -> Self {
        Self {
            clients: RwLock::new(HashSet::new()),
        }
    }

    pub fn is_registered(&self, client: &IpAddr) -> bool {
        self.clients.read().unwrap().contains(client)
    }

    pub fn register_client(&self, client: IpAddr) -> () {
        self.clients.write().unwrap().insert(client);
    }

    pub fn unregister_client(&self, client: IpAddr) -> () {
        self.clients.write().unwrap().remove(&client);
    }

    pub fn get(&self) -> Box<[IpAddr]> {
        self.clients
            .read()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<IpAddr>>()
            .into_boxed_slice()
    }
}
