use std::{borrow::Cow, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, thread};

use dashmap::DashSet;

use crate::{clients_registry::ClientsRegistry, common::{constants::*, types::*}};


pub struct RadioFrequency {
    pub freq: Option<u32>,
    pub channel: Channel,
    pub socket: UdpSocket,
    pub clients: ClientsRegistry,
    pub em_port: u16,
    pub rec_port: u16,
    pub pending_gps_clients: DashSet<IpAddr>
}


impl RadioFrequency {
    pub fn init(freq: Option<u32>) -> Self {
        let channel: Channel = if freq == Some(161975000) { Channel::C87B } else if freq == Some(161975001) { Channel::C88B } else { Channel::GPS };
        let em_port: u16 = if matches!(channel, Channel::C87B) { C87B_EM_PORT } else if matches!(channel, Channel::C88B) { C88B_EM_PORT } else { GPS_EM_PORT };
        let rec_port: u16 = if matches!(channel, Channel::C87B) { C87B_REC_PORT } else if matches!(channel, Channel::C88B) { C88B_REC_PORT } else { GPS_REC_PORT };

        Self {
            freq: freq,
            channel: channel,
            socket: UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)), rec_port)).unwrap(),
            clients: ClientsRegistry::init(),
            em_port: em_port,
            rec_port: rec_port,
            pending_gps_clients: DashSet::new()
        }
    }


    fn parse_transmission(size: usize, source: SocketAddr, buf: &[u8]) -> Cow<'_, str> {
        let msg: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&buf[..size]);
        println!("Reçu de {} : {}", source, msg);
        msg
    }


    pub fn relay(&self, msg: Cow<'_, str>) {
        for client_ip in self.clients.get() {
            let _ = self.socket.send_to(msg.as_bytes(), SocketAddr::new(client_ip, self.em_port));
        }
    }


    pub fn handle_gps_request(&self, client: IpAddr) -> () {
        let _ = self.socket.send_to(client.to_string().as_bytes(), SocketAddr::new(HARBOURMASTER_IP, GPS_EM_PORT));
        self.pending_gps_clients.insert(client);
    }


    pub fn handle_gps_response(&self, msg: Cow<'_, str>) -> () {
        let mut parts: std::str::SplitN<'_, &str> = msg.splitn(2, " | ");
        let client: IpAddr = parts.next().unwrap().parse().unwrap();
        let data: String = parts.next().unwrap().to_string();

        let _ = self.socket.send_to(data.as_bytes(), SocketAddr::new(client, GPS_EM_PORT));
        self.pending_gps_clients.remove(&client);
    }


    pub fn start(self) {
        thread::spawn(move || {
            let mut buf: [u8; 5096] = [0; 5096];

            loop {
                match self.socket.recv_from(&mut buf) {
                    Ok((size, source)) => {
                        self.clients.register_client(source.ip());

                        let msg: Cow<'_, str> = RadioFrequency::parse_transmission(size, source, &buf);

                        if matches!(self.channel, Channel::GPS) {
                            if source.ip() != HARBOURMASTER_IP {
                                self.handle_gps_request(source.ip());
                            } else {
                                self.handle_gps_response(msg);
                            }
                        } else if msg != "hello" {
                            self.relay(msg);
                        }
                    },
                    Err(e) => eprintln!("Erreur : {e}.")
                }
            }
        });
    }
}
