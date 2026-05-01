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
    is_simulation: bool,

    server_ip: IpAddr,
    harbourmaster_ip: IpAddr,

    cli: bool,
    gui: bool,
    wui: bool,

    api: bool,

    gps_detection: bool,
    gps_refresh_delay: u64,

    max_cli_logs_history_length: usize,
    cli_refresh_delay: u64,

    boat_sys_logs_filename: String,
    boat_ais_logs_filename: String,
    boat_gps_logs_filename: String,
    boat_satcom_logs_filename: String,
    boat_computer_logs_filename: String,

    harbourmaster_sys_logs_filename: String,
    harbourmaster_ais_logs_filename: String,
    harbourmaster_gps_logs_filename: String,
    harbourmaster_satcom_logs_filename: String,
    harbourmaster_computer_logs_filename: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            is_simulation: false,

            server_ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
            harbourmaster_ip: IpAddr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),

            cli: true,
            gui: true,
            wui: true,

            api: true,

            gps_detection: true,
            gps_refresh_delay: 5,

            max_cli_logs_history_length: 1000,
            cli_refresh_delay: 100,

            boat_ais_logs_filename: String::from("logs/boat_ais.log"),
            boat_sys_logs_filename: String::from("logs/boat_system.log"),
            boat_gps_logs_filename: String::from("logs/boat_gps.log"),
            boat_satcom_logs_filename: String::from("logs/boat_satcom.log"),
            boat_computer_logs_filename: String::from("logs/boat_computer.log"),

            harbourmaster_ais_logs_filename: String::from("logs/harbourmaster_ais.log"),
            harbourmaster_sys_logs_filename: String::from("logs/harbourmaster_system.log"),
            harbourmaster_gps_logs_filename: String::from("logs/harbourmaster_gps.log"),
            harbourmaster_satcom_logs_filename: String::from("logs/harbourmaster_satcom.log"),
            harbourmaster_computer_logs_filename: String::from("logs/harbourmaster_computer.log"),
        }
    }
}

impl Config {
    pub fn init(is_sim: bool, serv_ip: IpAddr, harbourmaster_ip: IpAddr) -> Self {
        let mut config: Config = Self::default();

        config.set_is_simulation(is_sim);
        config.set_server_ip(serv_ip);
        config.set_harbourmaster_ip(harbourmaster_ip);

        config
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

    pub fn log_files_names(&self) -> [&str; 10] {
        [
            self.boat_sys_logs_filename(),
            self.boat_ais_logs_filename(),
            self.boat_gps_logs_filename(),
            self.boat_satcom_logs_filename(),
            self.boat_computer_logs_filename(),
            self.harbourmaster_sys_logs_filename(),
            self.harbourmaster_ais_logs_filename(),
            self.harbourmaster_gps_logs_filename(),
            self.harbourmaster_satcom_logs_filename(),
            self.harbourmaster_computer_logs_filename(),
        ]
    }
}
