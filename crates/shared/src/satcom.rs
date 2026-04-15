use tokio::sync::mpsc::{Receiver, Sender};

use crate::{bitpacker::BitPacker, satcom_message::SatComMessage};

pub struct SatCom {
    reader_rx: Receiver<BitPacker>,
    sender_rx: Receiver<SatComMessage>,
    pub antenna_tx: Sender<BitPacker>,
    pub board_computer_tx: Sender<SatComMessage>,
}

impl SatCom {
    pub fn new(
        reader_rx: Receiver<BitPacker>,
        sender_rx: Receiver<SatComMessage>,
        antenna_tx: Sender<BitPacker>,
        computer_tx: Sender<SatComMessage>,
    ) -> Self {
        Self {
            reader_rx: reader_rx,
            sender_rx: sender_rx,
            antenna_tx: antenna_tx,
            board_computer_tx: computer_tx,
        }
    }

    pub async fn start(mut self) -> () {
        tokio::spawn(async move {
            tokio::select! {
                Some(msg) = self.reader_rx.recv() => {
                    self.board_computer_tx.send(SatComMessage::parse(msg).unwrap()).await;
                },
                Some(msg) = self.sender_rx.recv() => {
                    self.antenna_tx.send(msg.to_bitpacker()).await;
                }
            }
        });
    }
}
