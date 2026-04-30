use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

#[derive(Serialize, Deserialize, Debug, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct Config {
    server_ip: IpAddr,
    harbourmaster_ip: IpAddr,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_ip: IpAddr::V4(Ipv4Addr::from_str("0.0.0.0").unwrap()),
            harbourmaster_ip: IpAddr::V4(Ipv4Addr::from_str("0.0.0.0").unwrap()),
        }
    }
}

impl Config {
    pub fn init(serv_ip: IpAddr, harbourmaster_ip: IpAddr) -> Self {
        Self {
            server_ip: serv_ip,
            harbourmaster_ip: harbourmaster_ip,
        }
    }

    pub fn load() -> Option<Self> {
        match fs::read_to_string("./config.toml") {
            Ok(content) => toml::from_str(&content).unwrap_or(None),
            Err(_) => None,
        }
    }

    pub fn write(&self) -> () {
        let content = toml::to_string_pretty(self).expect("Erreur de sérialisation");
        fs::write("config.toml", content).expect("Impossible de sauvegarder la configuration");
    }
}
