use crate::{
    bitpacker::BitPacker,
    common::{errors::{AntennaError, AntennaResult}, types::{AisPacket, Channel}},
    config::Config,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub struct Antenna {
    channel: Channel,
    socket: UdpSocket,
    ant_rx: Receiver<BitPacker>,
    ais_tx: Option<Sender<AisPacket>>,
    gps_tx: Option<Sender<BitPacker>>,
    satcom_tx: Option<Sender<BitPacker>>,
    rec_port: u16,
    em_port: u16,
}

impl Antenna {
    pub async fn init(
        ais_tx: Option<Sender<AisPacket>>,
        gps_tx: Option<Sender<BitPacker>>,
        satcom_tx: Option<Sender<BitPacker>>,
        rx: Receiver<BitPacker>,
        em_port: u16,
        rec_port: u16,
        chn: Channel,
    ) -> AntennaResult<Self> {
        let sock: UdpSocket = UdpSocket::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            rec_port,
        ))
        .await?;

        Ok(Self {
            channel: chn,
            socket: sock,
            ant_rx: rx,
            ais_tx: ais_tx,
            gps_tx: gps_tx,
            satcom_tx: satcom_tx,
            rec_port: rec_port,
            em_port: em_port,
        })
    }

    pub async fn emit(&self, msg: BitPacker) -> AntennaResult<()> {
        let _ = self
            .socket
            .send_to(
                msg.bits(),
                SocketAddr::new(*Config::load().unwrap().server_ip(), self.em_port),
            )
            .await
            .map_err(|_| AntennaError::EmissionError);

        Ok(())
    }

    async fn run_listener(&mut self) -> AntennaResult<()> {
        let mut buf: [u8; 512] = [0; 512];

        loop {
            tokio::select! {
                result = self.socket.recv_from(&mut buf) => {
                    let (size, _source) = result.map_err(|_| AntennaError::ReceptionError)?;
                    let msg: BitPacker = BitPacker::from_slice(&buf[..size], Some(size * 8));

                    if msg.bits() != BitPacker::from_str("hello", None).bits() {
                        match self.channel {
                            Channel::C87B | Channel::C88B => {
                                self.ais_tx.clone().unwrap().send(AisPacket::from(msg, self.channel)).await?;
                            },
                            Channel::GPS => {
                                self.gps_tx.clone().unwrap().send(msg).await?;
                            },
                            Channel::SatCom => {
                                self.satcom_tx.clone().unwrap().send(msg).await?;
                            },
                            Channel::Any => todo!()
                        }
                    }
                },
                Some(msg) = self.ant_rx.recv() => {
                    self.emit(msg).await?;
                }
            }
        }
    }

    pub async fn start(mut self) -> AntennaResult<JoinHandle<()>> {
        self.emit(BitPacker::from_str("hello", None)).await?;

        Ok(tokio::spawn(async move {
            self.run_listener().await;
        }))
    }
}
