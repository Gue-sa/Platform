use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use dashmap::DashSet;

use tokio::net::UdpSocket;

use crate::{bitpacker::BitPacker, clients_registry::ClientsRegistry, common::{constants::*, types::*}};


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
    pub async fn init(freq: Option<u32>) -> Self {
        let channel: Channel = if freq == Some(161975000) { Channel::C87B } else if freq == Some(161975001) { Channel::C88B } else { Channel::GPS };
        let em_port: u16 = if matches!(channel, Channel::C87B) { C87B_EM_PORT } else if matches!(channel, Channel::C88B) { C88B_EM_PORT } else { GPS_EM_PORT };
        let rec_port: u16 = if matches!(channel, Channel::C87B) { C87B_REC_PORT } else if matches!(channel, Channel::C88B) { C88B_REC_PORT } else { GPS_REC_PORT };

        Self {
            freq: freq,
            channel: channel,
            socket: UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0,0,0,0)), rec_port)).await.unwrap(),
            clients: ClientsRegistry::init(),
            em_port: em_port,
            rec_port: rec_port,
            pending_gps_clients: DashSet::new()
        }
    }


    pub async fn relay(&self, buf: &[u8]) {
        for client_ip in self.clients.get() {
            let _ = self.socket.send_to(buf, SocketAddr::new(client_ip, self.em_port)).await;
        }
    }


    pub async fn handle_gps_request(&self, client: IpAddr) -> () {
        let _ = self.socket.send_to(client.to_string().as_bytes(), SocketAddr::new(HARBOURMASTER_IP, GPS_EM_PORT)).await;
        self.pending_gps_clients.insert(client);
    }


    pub async fn handle_gps_response(&self, msg: BitPacker) -> () {
        let client: IpAddr = IpAddr::V4(Ipv4Addr::from_bits(msg.extract_int::<u32>(None, Some(31)).unwrap()));
        let data: BitPacker = msg.slice(Some(32), None).unwrap();

        let _ = self.socket.send_to(data.bits(), SocketAddr::new(client, GPS_EM_PORT)).await;
        self.pending_gps_clients.remove(&client);
    }


    pub fn start(self) {
        tokio::spawn(async move {
            let mut buf: [u8; 512] = [0; 512];

            loop {
                let result = self.socket.recv_from(&mut buf).await;

                let (size, source) = result.unwrap();
                let msg: BitPacker = BitPacker::from_slice(&buf[..size], Some(size * 8 - 1)).unwrap();

                self.clients.register_client(source.ip());

                if matches!(self.channel, Channel::GPS) {
                    if source.ip() != HARBOURMASTER_IP {
                        self.handle_gps_request(source.ip()).await;
                    } else {
                        self.handle_gps_response(msg).await;
                    }
                } else if msg.bits() != BitPacker::from_str("hello", None).unwrap().bits() {
                    self.relay(&msg.bits()).await;
                }
            }
        });
    }
}
