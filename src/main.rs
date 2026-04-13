use std::thread::park;

use crate::{radio_frequency::RadioFrequency, shared::common::{constants::{C87B_EM_PORT, C87B_REC_PORT, C88B_EM_PORT, C88B_REC_PORT, GPS_EM_PORT, GPS_REC_PORT, SATCOM_EM_PORT, SATCOM_REC_PORT}, types::*}};


mod common;
mod radio_frequency;
mod clients_registry;
mod shared;


#[tokio::main]
async fn main() {
    let freq_c87b: RadioFrequency = RadioFrequency::init(Channel::C87B, C87B_EM_PORT, C87B_REC_PORT).await;
    let freq_c88b: RadioFrequency = RadioFrequency::init(Channel::C88B, C88B_EM_PORT, C88B_REC_PORT).await;
    let freq_gps: RadioFrequency = RadioFrequency::init(Channel::GPS, GPS_EM_PORT, GPS_REC_PORT).await;
    let freq_satcom: RadioFrequency = RadioFrequency::init(Channel::SATCOM, SATCOM_EM_PORT, SATCOM_REC_PORT).await;
    
    freq_c87b.start();
    freq_c88b.start();
    freq_gps.start();
    freq_satcom.start();

    park();
}
