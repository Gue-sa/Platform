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
    let boat: Boat = Boat::init();
    
    boat.start();

    loop{}

    //let heh1: BitPacker = BitPacker::from_str("a", None).unwrap();
    //let heh2: BitPacker = BitPacker::from_str("a", None).unwrap();
    //let heh: BitPacker = heh1 + heh2;
    
    //println!("{:?}", heh.extract_int::<i128>(None, None).unwrap());
}
