use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tokio::{net::UdpSocket, sync::mpsc::{Receiver, Sender}};

use crate::{common::{constants::*, types::*}, shared::bitpacker::BitPacker};


pub struct Antenna {
    pub freq: Option<u32>,
    pub channel: Channel,
    pub socket: UdpSocket,
    ant_rx: Receiver<Packet>,
    pub station_tx: Sender<Packet>,
    rec_port: u16,
    em_port: u16
}


impl Antenna {
    pub async fn init(freq: Option<u32>, station_tx: Sender<Packet>, rx: Receiver<Packet>) -> Self {
        let channel: Channel = if freq == Some(161975000) { Channel::C87B } else if freq == Some(161975001) { Channel::C88B } else { Channel::GPS };
        let rec_port: u16 = if matches!(channel, Channel::C87B) { C87B_EM_PORT } else if matches!(channel, Channel::C88B) { C88B_EM_PORT } else { GPS_EM_PORT };
        let em_port: u16 = if matches!(channel, Channel::C87B) { C87B_REC_PORT } else if matches!(channel, Channel::C88B) { C88B_REC_PORT } else { GPS_REC_PORT };
        let socket: UdpSocket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)), rec_port)).await.unwrap();
        
        Self {
            freq: freq,
            channel: channel,
            socket: socket,
            ant_rx: rx,
            station_tx: station_tx,
            rec_port: rec_port,
            em_port: em_port
        }
    }


    pub async fn send(&self, msg: Packet) -> () {
        let _ = self.socket.send_to(msg.message.bits(), SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 1)), self.em_port)).await;
    }


    pub async fn start(mut self) -> () {
        self.send(Packet { message: BitPacker::from_str("hello", None).unwrap(), channel: Channel::C87B}).await;

        tokio::spawn(async move {
            let mut buf: [u8; 512] = [0; 512];

            loop {
                tokio::select! {
                    result = self.socket.recv_from(&mut buf) => {
                        let (size, source) = result.unwrap();
                        let msg: BitPacker = BitPacker::from_slice(&buf[..size], Some(size * 8)).unwrap();

                        if msg.bits() != BitPacker::from_str("hello", None).unwrap().bits() {
                            let packet: Packet = Packet {
                                message: BitPacker::from_slice(&buf[..size], Some(size * 8)).unwrap(),
                                channel: self.channel
                            };

                            let _ = self.station_tx.send(packet).await;
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