use std::{io, net::{ IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, sync::{Arc, Mutex, mpsc::{Receiver, Sender, channel}}, thread};

use local_ip_address::list_afinet_netifas;

use crate::common::{constants::*, types::*};


pub struct Packet {
    pub channel: Channel,
    pub message: String
}


pub struct Antenna {
    pub freq: Option<u32>,
    pub channel: Channel,
    pub socket: UdpSocket,
    pub ant_tx: Sender<String>,
    ant_rx: Mutex<Receiver<String>>,
    pub ais_tx: Option<Sender<Packet>>,
    pub gps_tx: Option<Sender<String>>,
    rec_port: u16,
    em_port: u16
}


impl Packet {
    pub fn from(msg: String, chn: Channel) -> Self {
        Self {
            channel: chn,
            message: msg
        }
    }
}


impl Antenna {
    pub fn init(freq: Option<u32>, ais_tx: Option<Sender<Packet>>, gps_tx: Option<Sender<String>>, tx: Sender<String>, rx: Receiver<String>) -> Self {
        let channel: Channel = if freq == Some(161975000) { Channel::C87B } else if freq == Some(161975001) { Channel::C88B } else { Channel::GPS };
        let em_port: u16 = if matches!(channel, Channel::C87B) { C87B_REC_PORT } else if matches!(channel, Channel::C88B) { C88B_REC_PORT } else { GPS_REC_PORT };
        let rec_port: u16 = if matches!(channel, Channel::C87B) { C87B_EM_PORT } else if matches!(channel, Channel::C88B) { C88B_EM_PORT } else { GPS_EM_PORT };
        let socket: UdpSocket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)), rec_port)).unwrap();
        let _ = socket.set_nonblocking(true);
        
        Self {
            freq: freq,
            channel: if freq == Some(161975000) { Channel::C87B } else if freq == Some(161975001) { Channel::C88B } else { Channel::GPS },
            socket: socket,
            ant_tx: tx,
            ant_rx: Mutex::new(rx),
            ais_tx: ais_tx,
            gps_tx: gps_tx,
            rec_port: rec_port,
            em_port: em_port
        }
    }


    pub fn start(self: Arc<Self>) -> () {
        self.send(String::from("hello"));

        thread::spawn(move || {
            loop {
                if let Ok(msg) = self.ant_rx.lock().unwrap().try_recv() {
                    self.send(msg);
                }

                if let Some(msg) = self.listen() {
                    match self.channel {
                        Channel::C87B | Channel::C88B => {
                            let _ = self.ais_tx.clone().unwrap().send(Packet::from(msg, self.channel));
                        },
                        Channel::GPS => {
                            let _ = self.gps_tx.clone().unwrap().send(msg);
                        },
                        Channel::Any => todo!()
                    }
                }
            }
        });
    }


    pub fn listen(&self) -> Option<String> {
        let mut buf: [u8; 5096] = [0; 5096];

        match self.socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                Some(String::from_utf8_lossy(&buf[..size]).into_owned())
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => None,
            Err(e) => {
                eprintln!("Erreur : {e}.");
                None
            }
        }
    }


    pub fn send(&self, msg: String) -> () {
        //let server_ip: IpAddr = *list_afinet_netifas().unwrap().iter().find(|(nom, _)| nom == "wlan0").map(|(_, ip)| ip).unwrap();
        let server_ip: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        let _ = self.socket.send_to(msg.as_bytes(), SocketAddr::new(server_ip, self.em_port));
    }
}