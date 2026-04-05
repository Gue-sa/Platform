use crate::{boat::Boat, common::bitpacker::BitPacker};

mod common;
mod ais;
mod antenna;
mod boat;
mod message;
mod slot;
mod slots_map;
mod boat_info;
mod boats_registry;
mod gps;
mod display;


fn main() {
    //let boat: Boat = Boat::init();
    
    //boat.start();

    //loop{}

    let boxe: Box<[u8]> = Box::<[u8]>::from([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 248, 164, 50, 235]);
    //let mut heh1: BitPacker<String> = BitPacker::from_str("a", None).unwrap();
    //let heh2: BitPacker<String> = BitPacker::from_str("a", None).unwrap();
    //let heh: BitPacker<String> = BitPacker::<String>::parse_str(boxe).unwrap();
    let mut heh1: BitPacker<u8> = BitPacker::from_int(1, None).unwrap();
    let heh2: BitPacker<u8> = BitPacker::from_int(1, None).unwrap();
    //let heh: BitPacker<i128> = BitPacker::parse_int(boxe).unwrap();
    let _ = heh1.concat_int(heh2);
    println!("{:?}", heh1);
}
