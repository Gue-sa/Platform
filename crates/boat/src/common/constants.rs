use std::net::{IpAddr, Ipv4Addr};

pub const BOAT_IPV4: Ipv4Addr = Ipv4Addr::new(10, 0, 1, 2);
pub const BOAT_IP: IpAddr = IpAddr::V4(BOAT_IPV4);
