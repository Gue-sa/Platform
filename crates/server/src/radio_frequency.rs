use dashmap::DashSet;
use shared::{
    bitpacker::BitPacker,
    clients_registry::ClientsRegistry,
    common::{constants::GPS_FROM_SERVER_PORT, errors::RadioFrequencyResult, types::Channel},
    config::Config,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::{net::UdpSocket, task::JoinHandle};

pub struct RadioFrequency {
    channel: Channel,
    socket: UdpSocket,
    clients: ClientsRegistry,
    em_port: u16,
    rec_port: u16,
    pending_gps_clients: DashSet<IpAddr>,
}

impl RadioFrequency {
    pub async fn init(chn: Channel, em_port: u16, rec_port: u16) -> Self {
        Self {
            channel: chn,
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

    async fn relay(&self, buf: &[u8]) -> RadioFrequencyResult<()> {
        for client_ip in self.clients.get()? {
            self.socket
                .send_to(buf, SocketAddr::new(client_ip, self.em_port))
                .await;
        }

        Ok(())
    }

    async fn handle_gps_request(&self, msg: BitPacker) -> RadioFrequencyResult<()> {
        self.socket
            .send_to(
                msg.bits(),
                SocketAddr::new(
                    *Config::load().unwrap().harbourmaster_ip(),
                    GPS_FROM_SERVER_PORT,
                ),
            )
            .await;
        self.pending_gps_clients
            .insert(IpAddr::V4(Ipv4Addr::from_bits(
                msg.extract_int::<u32>(None, None)?,
            )));

        Ok(())
    }

    async fn handle_gps_response(&self, msg: BitPacker) -> RadioFrequencyResult<()> {
        let client = IpAddr::V4(Ipv4Addr::from_bits(msg.extract_int::<u32>(None, Some(31))?));

        let data = msg.slice(Some(32), None)?;

        self.socket
            .send_to(data.bits(), SocketAddr::new(client, GPS_FROM_SERVER_PORT))
            .await;
        self.pending_gps_clients.remove(&client);

        Ok(())
    }

    pub fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut buf: [u8; 512] = [0; 512];

            loop {
                if let Ok((size, source)) = self.socket.recv_from(&mut buf).await {
                    let msg = BitPacker::from_slice(&buf[..size], Some(size * 8));

                    println!("{}: {}\n", source, msg.to_bin_str());

                    self.clients.register_client(source.ip());

                    if msg.bits() != BitPacker::from_str("hello", None).bits() {
                        if matches!(self.channel, Channel::GPS) {
                            if source.ip() != *Config::load().unwrap().harbourmaster_ip() {
                                self.handle_gps_request(msg).await;
                            } else {
                                self.handle_gps_response(msg).await;
                            }
                        } else {
                            self.relay(&msg.bits()).await;
                        }
                    }
                }
            }
        })
    }
}
