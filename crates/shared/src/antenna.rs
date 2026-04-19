use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender},
};

use crate::{
    bitpacker::BitPacker,
    common::types::{AisPacket, Channel},
};

pub struct Antenna {
    pub channel: Channel,
    pub socket: UdpSocket,
    ant_rx: Receiver<BitPacker>,
    pub ais_tx: Option<Sender<AisPacket>>,
    pub gps_tx: Option<Sender<BitPacker>>,
    pub satcom_tx: Option<Sender<BitPacker>>,
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
        channel: Channel,
    ) -> Self {
        let socket: UdpSocket = UdpSocket::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            rec_port,
        ))
        .await
        .unwrap();

        Self {
            channel: channel,
            socket: socket,
            ant_rx: rx,
            ais_tx: ais_tx,
            gps_tx: gps_tx,
            satcom_tx: satcom_tx,
            rec_port: rec_port,
            em_port: em_port,
        }
    }

    pub async fn send(&self, msg: BitPacker) -> () {
        //let server_ip: IpAddr = *list_afinet_netifas().unwrap().iter().find(|(nom, _)| nom == "wlan0").map(|(_, ip)| ip).unwrap();
        let server_ip: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        self
            .socket
            .send_to(msg.bits(), SocketAddr::new(server_ip, self.em_port))
            .await
            .unwrap();
    }

    pub async fn start(mut self) -> () {
        self.send(BitPacker::from_str("hello", None)).await;

        tokio::spawn(async move {
            let mut buf: [u8; 512] = [0; 512];

            loop {
                tokio::select! {
                    result = self.socket.recv_from(&mut buf) => {
                        let (size, source) = result.unwrap();
                        let msg: BitPacker = BitPacker::from_slice(&buf[..size], Some(size * 8));

                        if msg.bits() != BitPacker::from_str("hello", None).bits() {
                            match self.channel {
                                Channel::C87B | Channel::C88B => {
                                    self.ais_tx.clone().unwrap().send(AisPacket::from(msg, self.channel)).await;
                                },
                                Channel::GPS => {
                                    self.gps_tx.clone().unwrap().send(msg).await;
                                },
                                Channel::SATCOM => {
                                    self.satcom_tx.clone().unwrap().send(msg).await;
                                },
                                Channel::Any => todo!()
                            }
                        }
                    },
                    Some(msg) = self.ant_rx.recv() => {
                        self.send(msg).await;
                    }
                }
            }
        });
    }
}
