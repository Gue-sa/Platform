use std::{sync::{Arc, Mutex, mpsc::{Receiver, Sender, channel}}, thread};

use crate::{antenna::Antenna, common::types::Packet};

pub struct Gps {
    pub tx: Sender<Packet>,
    rx: Mutex<Receiver<Packet>>,
    pub antenna_tx: Sender<Packet>,
    pub antenna: Arc<Antenna>
}


impl Gps {
    pub fn init() -> Self {
        let (tx, rx) = channel::<Packet>();
        let (antenna_tx, antenna_rx) = channel::<Packet>();
        let antenna: Antenna = Antenna::init(None, tx.clone(), antenna_tx.clone(), antenna_rx);

        Self {
            tx: tx.clone(),
            rx: Mutex::new(rx),
            antenna_tx: antenna_tx.clone(),
            antenna: Arc::new(antenna)
        }
    }


    pub fn handle_request(&self, msg_packet: Packet) -> () {
        let res_packet: Packet = Packet {
            message: format!("{} | 0 | 0", msg_packet.message),
            channel: msg_packet.channel,
            client: msg_packet.client
        };

        println!("Requête GPS reçue : {:?}", res_packet.message);

        let _ = self.antenna_tx.send(res_packet);
    }


    pub fn listen(self: Arc<Self>) -> () {
        thread::spawn(move || {
            loop {
                if let Ok(rx_guard) = self.rx.lock() {
                    for packet in rx_guard.try_iter() {
                        self.handle_request(packet);
                    }
                }
            }
        });
    }


    pub fn start(self: Arc<Self>) -> () {
        Arc::clone(&self.antenna).start();
        self.listen();
    }
}