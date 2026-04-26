use crate::{bitpacker::BitPacker, satcom_message::SatComMessage};
use tokio::{sync::mpsc::{Receiver, Sender}, task::JoinHandle};

pub struct SatCom {
    reader_rx: Receiver<BitPacker>,
    sender_rx: Receiver<SatComMessage>,
    antenna_tx: Sender<BitPacker>,
    board_computer_tx: Sender<SatComMessage>,
}

impl SatCom {
    pub fn init(
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

    async fn satcom_runner(&mut self) -> () {
        loop {
            tokio::select! {
                Some(msg) = self.reader_rx.recv() => {
                    if let Ok(parsed_msg) = SatComMessage::parse(msg) {
                        self.board_computer_tx.send(parsed_msg).await;
                    }
                },
                Some(msg) = self.sender_rx.recv() => {
                    self.antenna_tx.send(msg.to_bitpacker()).await;
                }
            }
        }
    }

    pub fn start(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.satcom_runner().await;
        })
    }
}
