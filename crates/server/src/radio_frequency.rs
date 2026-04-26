use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use dashmap::DashSet;

use shared::{
    bitpacker::BitPacker,
    common::{constants::GPS_EM_PORT, types::Channel},
};
use tokio::net::UdpSocket;

use crate::{clients_registry::ClientsRegistry, common::constants::HARBOURMASTER_IP};

pub struct RadioFrequency {
    channel: Channel,
    socket: UdpSocket,
    clients: ClientsRegistry,
    em_port: u16,
    rec_port: u16,
    pending_gps_clients: DashSet<IpAddr>,
}

impl RadioFrequency {
    pub async fn init(channel: Channel, em_port: u16, rec_port: u16) -> Self {
        Self {
            channel: channel,
            socket: UdpSocket::bind(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                rec_port,
            ))
            .await
            .unwrap(),
            clients: ClientsRegistry::init(),
            em_port: em_port,
            rec_port: rec_port,
            pending_gps_clients: DashSet::new(),
        }
    }

    async fn relay(&self, buf: &[u8]) {
        for client_ip in self.clients.get() {
            self
                .socket
                .send_to(buf, SocketAddr::new(client_ip, self.em_port))
                .await;
        }
    }

    async fn handle_gps_request(&self, msg: BitPacker) -> () {
        self
            .socket
            .send_to(msg.bits(), SocketAddr::new(HARBOURMASTER_IP, GPS_EM_PORT))
            .await;
        self.pending_gps_clients
            .insert(IpAddr::V4(Ipv4Addr::from_bits(
                msg.extract_int::<u32>(None, None).unwrap(),
            )));
    }

    async fn handle_gps_response(&self, msg: BitPacker) -> () {
        let client: IpAddr = IpAddr::V4(Ipv4Addr::from_bits(
            msg.extract_int::<u32>(None, Some(31)).unwrap(),
        ));

        let data: BitPacker = msg.slice(Some(32), None).unwrap();

        self
            .socket
            .send_to(data.bits(), SocketAddr::new(client, GPS_EM_PORT))
            .await;
        self.pending_gps_clients.remove(&client);
    }

    pub fn start(self) {
        tokio::spawn(async move {
            let mut buf: [u8; 512] = [0; 512];

            loop {
                let result = self.socket.recv_from(&mut buf).await;

                let (size, source) = result.unwrap();
                let msg: BitPacker = BitPacker::from_slice(&buf[..size], Some(size * 8));

                println!("{}: {}\n", source, msg.to_bin_string());

                self.clients.register_client(source.ip());

                if msg.bits() != BitPacker::from_str("hello", None).bits() {
                    if matches!(self.channel, Channel::GPS) {
                        if source.ip() != HARBOURMASTER_IP {
                            self.handle_gps_request(msg).await;
                        } else {
                            self.handle_gps_response(msg).await;
                        }
                    } else {
                        self.relay(&msg.bits()).await;
                    }
                }
            }
        });
    }
}
