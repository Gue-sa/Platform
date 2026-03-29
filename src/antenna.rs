use std::{io, net::{ IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, sync::{Arc, Mutex, mpsc::{Receiver, Sender}}, thread};

use crate::common::{constants::*, types::*};


pub struct Antenna {
    pub freq: Option<u32>,
    pub channel: Channel,
    pub socket: UdpSocket,
    pub ant_tx: Sender<Packet>,
    ant_rx: Mutex<Receiver<Packet>>,
    pub station_tx: Sender<Packet>,
    rec_port: u16,
    em_port: u16
}


impl Antenna {
    pub fn init(freq: Option<u32>, station_tx: Sender<Packet>, tx: Sender<Packet>, rx: Receiver<Packet>) -> Self {
        let channel: Channel = if freq == Some(161975000) { Channel::C87B } else if freq == Some(161975001) { Channel::C88B } else { Channel::GPS };
        let rec_port: u16 = if matches!(channel, Channel::C87B) { C87B_EM_PORT } else if matches!(channel, Channel::C88B) { C88B_EM_PORT } else { GPS_EM_PORT };
        let em_port: u16 = if matches!(channel, Channel::C87B) { C87B_REC_PORT } else if matches!(channel, Channel::C88B) { C88B_REC_PORT } else { GPS_REC_PORT };
        let socket: UdpSocket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)), rec_port)).unwrap();
        let _ = socket.set_nonblocking(true);
        
        Self {
            freq: freq,
            channel: channel,
            socket: socket,
            ant_tx: tx,
            ant_rx: Mutex::new(rx),
            station_tx: station_tx,
            rec_port: rec_port,
            em_port: em_port
        }
    }


    pub fn listen(&self) -> Option<Packet> {
        let mut buf: [u8; 5096] = [0; 5096];

        match self.socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let packet: Packet = Packet {
                    message: String::from_utf8_lossy(&buf[..size]).into_owned(),
                    channel: self.channel,
                    client: source.ip()
                };
                Some(packet)
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => None,
            Err(e) => {
                eprintln!("Erreur : {e}.");
                None
            }
        }
    }


    pub fn send(&self, msg: Packet) -> () {
        let _ = self.socket.send_to(msg.message.as_bytes(), SocketAddr::new(msg.client, self.em_port));
    }


    pub fn start(self: Arc<Self>) -> () {
        self.send(Packet { message: String::from("hello"), channel: Channel::C87B, client: IpAddr::V4(Ipv4Addr::new(10,0,0,2)) });

        thread::spawn(move || {
            loop {
                if let Ok(msg) = self.ant_rx.lock().unwrap().try_recv() {
                    self.send(msg);
                }

                if let Some(msg_packet) = self.listen() {
                    let _ = self.station_tx.send(msg_packet);
                }
            }
        });
    }
}