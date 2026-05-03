use crate::{bitpacker::BitPacker, common::types::LogEvent, satcom_message::SatComMessage};
use colored::Colorize;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub struct SatCom {
    reader_rx: Receiver<BitPacker>,
    sender_rx: Receiver<SatComMessage>,
    antenna_tx: Sender<BitPacker>,
    board_computer_tx: Sender<SatComMessage>,
    logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
    mmsi: u32,
}

impl SatCom {
    pub fn init(
        reader_rx: Receiver<BitPacker>,
        sender_rx: Receiver<SatComMessage>,
        antenna_tx: Sender<BitPacker>,
        computer_tx: Sender<SatComMessage>,
        cli_tx: std::sync::mpsc::Sender<LogEvent>,
        mmsi: u32,
    ) -> Self {
        Self {
            reader_rx: reader_rx,
            sender_rx: sender_rx,
            antenna_tx: antenna_tx,
            board_computer_tx: computer_tx,
            logs_cli_tx: cli_tx,
            mmsi: mmsi,
        }
    }

    fn logs_cli_tx(&self) -> std::sync::mpsc::Sender<LogEvent> {
        self.logs_cli_tx.clone()
    }

    async fn satcom_runner(&mut self) -> () {
        loop {
            tokio::select! {
                Some(msg) = self.reader_rx.recv() => {
                    if let Ok(parsed_msg) = SatComMessage::parse(&msg) {
                        self.board_computer_tx.send(parsed_msg.clone()).await;

                        if *parsed_msg.source() != self.mmsi {
                            self.logs_cli_tx().send(LogEvent::Satcom(format!(
                                "Message SatCom reçu : {:?}", msg.bits()
                            ).green()));
                        }
                    } else {
                        self.logs_cli_tx().send(LogEvent::Satcom(format!(
                            "Message SatCom malformé reçu et ignoré : {:?}", msg.bits()
                        ).red()));
                    }
                },
                Some(msg) = self.sender_rx.recv() => {
                    self.antenna_tx.send(msg.to_bitpacker()).await;

                    self.logs_cli_tx().send(LogEvent::Satcom(format!(
                        "Message SatCom envoyé : {:?}", msg.to_bitpacker().bits()
                    ).green()));
                }
            }
        }
    }

    pub fn start(mut self) -> JoinHandle<()> {
        self.logs_cli_tx().send(LogEvent::System(
            "Démarrage des communications satellite (SatCom)...".yellow(),
        ));

        tokio::spawn(async move {
            self.satcom_runner().await;
        })
    }
}
