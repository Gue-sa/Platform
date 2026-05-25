use colored::Colorize;
use getset::{Getters, Setters};
use serialport::{SerialPortType, available_ports};
use shared::common::types::LogEvent;
use slint::format;
use std::{
    sync::{Arc, RwLock},
    thread::sleep,
    time::Duration,
};
use tokio::task::JoinHandle;

#[derive(Getters, Setters)]
#[getset(get = "pub", set = "pub")]
struct MotorsConfig {
    left_motor_power_percentage: i8,
    right_motor_percentage: i8,
}

pub struct SerialDriver {
    logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
    motors_config: Arc<RwLock<MotorsConfig>>,
}

impl SerialDriver {
    pub fn init(logs_cli_tx: std::sync::mpsc::Sender<LogEvent>) -> Self {
        Self {
            motors_config: Arc::new(RwLock::new(MotorsConfig {
                left_motor_power_percentage: 0,
                right_motor_percentage: 0,
            })),
            logs_cli_tx: logs_cli_tx,
        }
    }

    pub fn change_motors_config(&mut self, left: Option<i8>, right: Option<i8>) {
        if let Some(l) = left {
            self.motors_config
                .write()
                .unwrap()
                .set_left_motor_power_percentage(l);
        }

        if let Some(r) = right {
            self.motors_config
                .write()
                .unwrap()
                .set_right_motor_percentage(r);
        }

        self.logs_cli_tx.send(LogEvent::System(
            format!(
                "Nouvelle configuration moteurs: droit: {}%, gauche: {}%",
                self.motors_config.read().unwrap().right_motor_percentage,
                self.motors_config
                    .read()
                    .unwrap()
                    .left_motor_power_percentage
            )
            .green(),
        ));
    }

    pub fn start(&self) -> JoinHandle<()> {
        let ports: Vec<serialport::SerialPortInfo> = available_ports().unwrap();

        let port_name = ports
            .into_iter()
            .find_map(|p| {
                if let SerialPortType::UsbPort(_) = p.port_type {
                    Some(p.port_name)
                } else {
                    None
                }
            })
            .unwrap();

        let mut port = serialport::new(port_name, 115_200)
            .timeout(Duration::from_millis(100))
            .open()
            .unwrap();

        let motors_config_clone = self.motors_config.clone();

        tokio::spawn(async move {
            loop {
                let order = format!(
                    "left-right {} {}",
                    motors_config_clone
                        .read()
                        .unwrap()
                        .left_motor_power_percentage(),
                    motors_config_clone.read().unwrap().right_motor_percentage()
                )
                .to_string();

                port.write_all(order.as_bytes());

                sleep(Duration::from_millis(100));
            }
        })

        /*
        let mut buffer = vec![0; 32];
        if let Ok(bytes_read) = port.read(buffer.as_mut_slice()) {
            println!("Reçu : {:?}", &buffer[..bytes_read]);
        }
        */
    }
}
