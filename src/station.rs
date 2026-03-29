use std::{sync::{Arc, Mutex, mpsc::{Receiver, Sender, channel}}, thread};

use crate::{antenna::Antenna, common::types::{Channel, Packet}};

pub struct Station {
    pub harbourmaster_tx: Sender<Packet>,
    pub c87b_tx: Sender<Packet>,
    pub c88b_tx: Sender<Packet>,
    pub tx: Sender<Packet>,
    rx: Mutex<Receiver<Packet>>,
    pub c87b_antenna: Arc<Antenna>,
    pub c88b_antenna: Arc<Antenna>
}


impl Station {
    pub fn init(tx: Sender<Packet>, rx: Receiver<Packet>, harbourmaster_tx: Sender<Packet>) -> Self {
        let (c87b_tx, c87b_rx) = channel::<Packet>();
        let (c88b_tx, c88b_rx) = channel::<Packet>();
        
        let c87b_antenna: Antenna = Antenna::init(Some(161975000), tx.clone(), c87b_tx.clone(), c87b_rx);
        let c88b_antenna: Antenna = Antenna::init(Some(161975001), tx.clone(), c88b_tx.clone(), c88b_rx);

        Self {
            harbourmaster_tx: harbourmaster_tx.clone(),
            c87b_tx: c87b_tx.clone(),
            c88b_tx: c88b_tx.clone(),
            tx: tx.clone(),
            rx: Mutex::new(rx),
            c87b_antenna: Arc::new(c87b_antenna),
            c88b_antenna: Arc::new(c88b_antenna)
        }
    }


    pub fn listen(self: Arc<Self>) -> () {
        thread::spawn(move || {
            loop {
                if let Ok(rx_guard) = self.rx.lock() {
                    for packet in rx_guard.try_iter() {
                        let _ = self.harbourmaster_tx.send(packet);
                    }  
                }
            }
        });
    }


    pub fn send(&self, msg: Packet) -> () {
        match msg.channel {
            Channel::C87B => {
                let _ = self.c87b_tx.send(msg);
            },
            Channel::C88B => {
                let _ = self.c88b_tx.send(msg);
            },
            _ => {}
        }
    }


    pub fn start(self: Arc<Self>) -> () {
        Arc::clone(&self.c87b_antenna).start();
        Arc::clone(&self.c88b_antenna).start();
        Arc::clone(&self).listen();
    }
}