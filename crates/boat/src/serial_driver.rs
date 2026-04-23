use std::{thread, time::Duration};

use serialport::{SerialPortType, available_ports};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct SerialDriver {
    //rx: Receiver<String>,
    //tx: Sender<String>,
}

impl SerialDriver {
    pub fn init(/*rx: Receiver<String>, tx: Sender<String>*/) -> Self {
        SerialDriver {
            //rx: rx,
            //tx: tx,
        }
    }

    pub async fn start(mut self) -> () {
        let ports = available_ports().unwrap();

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

        let mut port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(100))
            .open()
            .unwrap();

        tokio::spawn(async move {
            loop {
                port.write_all(b"1\n").unwrap();
                println!("Allumage.");

                thread::sleep(Duration::from_secs(1));

                port.write_all(b"0\n").unwrap();
                println!("Extinction.");

                thread::sleep(Duration::from_secs(1));
            }
        });

        /*
        let mut buffer: Vec<u8> = vec![0; 32];
        if let Ok(bytes_read) = port.read(buffer.as_mut_slice()) {
            println!("Reçu : {:?}", &buffer[..bytes_read]);
        }
        */
    }

    pub async fn set_speed(&mut self, speed: u16) {
        //self.tx.send(format!("SPEED:{}", speed)).await.unwrap();
    }

    pub async fn set_haeding(&mut self, heading: u16) {
        //self.tx.send(format!("HEADING:{}", heading)).await.unwrap();
    }

    pub async fn cross_distance(&mut self, distance: u16) {
        //self.tx.send(format!("CROSS_DISTANCE:{}", distance)).await.unwrap();
    }
}
