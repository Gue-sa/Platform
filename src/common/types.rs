use std::net::IpAddr;

#[derive(Copy, Clone)]
pub enum Channel {
    C87B,
    C88B,
    GPS,
    Any
}


pub struct Packet {
    pub message: String,
    pub channel: Channel,
    pub client: IpAddr
}