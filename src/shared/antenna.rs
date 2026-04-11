use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, sync::{Arc, Mutex}};

use tokio::{net::UdpSocket, sync::mpsc::{Receiver, Sender}};

use crate::{common::{constants::*, types::*}, shared::{bitpacker::BitPacker, common::{constants::{C87B_EM_PORT, C87B_REC_PORT, C88B_EM_PORT, C88B_REC_PORT, GPS_EM_PORT, GPS_REC_PORT}, types::Channel}}};


pub struct AisPacket {
    pub channel: Channel,
    pub message: BitPacker
}


pub struct Antenna {
    pub freq: Option<u32>,
    pub channel: Channel,
    pub socket: UdpSocket,
    ant_rx: Receiver<BitPacker>,
    pub ais_tx: Option<Sender<AisPacket>>,
    pub gps_tx: Option<Sender<BitPacker>>,
    rec_port: u16,
    em_port: u16
}


impl AisPacket {
    pub fn from(msg: BitPacker, chn: Channel) -> Self {
        Self {
            channel: chn,
            message: msg
        }
    }
}


impl Antenna {
    pub async fn init(freq: Option<u32>, ais_tx: Option<Sender<AisPacket>>, gps_tx: Option<Sender<BitPacker>>, rx: Receiver<BitPacker>) -> Self {
        let channel: Channel = if freq == Some(161975000) { Channel::C87B } else if freq == Some(161975001) { Channel::C88B } else { Channel::GPS };
        let em_port: u16 = if matches!(channel, Channel::C87B) { C87B_REC_PORT } else if matches!(channel, Channel::C88B) { C88B_REC_PORT } else { GPS_REC_PORT };
        let rec_port: u16 = if matches!(channel, Channel::C87B) { C87B_EM_PORT } else if matches!(channel, Channel::C88B) { C88B_EM_PORT } else { GPS_EM_PORT };
        let socket: UdpSocket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)), rec_port)).await.unwrap();
        
        Self {
            freq: freq,
            channel: if freq == Some(161975000) { Channel::C87B } else if freq == Some(161975001) { Channel::C88B } else { Channel::GPS },
            socket: socket,
            ant_rx: rx,
            ais_tx: ais_tx,
            gps_tx: gps_tx,
            rec_port: rec_port,
            em_port: em_port
        }
    }


    pub async fn send(&self, msg: BitPacker) -> () {
        //let server_ip: IpAddr = *list_afinet_netifas().unwrap().iter().find(|(nom, _)| nom == "wlan0").map(|(_, ip)| ip).unwrap();
        let server_ip: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 1, 1));
        let _ = self.socket.send_to(msg.bits(), SocketAddr::new(server_ip, self.em_port)).await.unwrap();
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
                                    let _ = self.ais_tx.clone().unwrap().send(AisPacket::from(msg, self.channel)).await;
                                },
                                Channel::GPS => {
                                    let _ = self.gps_tx.clone().unwrap().send(msg).await;
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