use crate::{radio_frequency::RadioFrequency};


mod common;
mod radio_frequency;
mod clients_registry;


fn main() {
    let freq_c87b: RadioFrequency = RadioFrequency::init(Some(161975000));
    let freq_c88b: RadioFrequency = RadioFrequency::init(Some(161975001));
    let freq_gps: RadioFrequency = RadioFrequency::init(None);
    freq_c87b.start();
    freq_c88b.start();
    freq_gps.start();

    loop {
        
    }
}
