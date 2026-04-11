use crate::{antenna::Antenna, common::types::{Channel, Packet}};

use tokio::sync::{Semaphore, mpsc::{Receiver, Sender, channel}};

pub struct Station {
    pub harbourmaster_tx: Sender<Packet>,
    pub c87b_tx: Sender<Packet>,
    pub c88b_tx: Sender<Packet>,
    rx: Receiver<Packet>,
    pub c87b_antenna: Antenna,
    pub c88b_antenna: Antenna
}


impl Station {
    pub async fn init(tx: Sender<Packet>, rx: Receiver<Packet>, harbourmaster_tx: Sender<Packet>) -> Self {
        let (c87b_tx, c87b_rx) = channel::<Packet>(Semaphore::MAX_PERMITS);
        let (c88b_tx, c88b_rx) = channel::<Packet>(Semaphore::MAX_PERMITS);
        
        let c87b_antenna: Antenna = Antenna::init(Some(161975000), tx.clone(), c87b_rx).await;
        let c88b_antenna: Antenna = Antenna::init(Some(161975001), tx.clone(), c88b_rx).await;

        Self {
            harbourmaster_tx: harbourmaster_tx,
            c87b_tx: c87b_tx.clone(),
            c88b_tx: c88b_tx.clone(),
            rx: rx,
            c87b_antenna: c87b_antenna,
            c88b_antenna: c88b_antenna
        }
    }


    pub async fn send(&self, msg: Packet) -> () {
        match msg.channel {
            Channel::C87B => {
                let _ = self.c87b_tx.send(msg).await;
            },
            Channel::C88B => {
                let _ = self.c88b_tx.send(msg).await;
            },
            _ => {}
        }
    }


    pub async fn start(mut self) -> () {
        let _ = &self.c87b_antenna.start().await;
        let _ = &self.c88b_antenna.start().await;

        tokio::spawn(async move {
            loop {
                let msg: Option<Packet> = self.rx.recv().await;

                let _ = self.harbourmaster_tx.send(msg.unwrap()).await;
            }
        });
    }
}