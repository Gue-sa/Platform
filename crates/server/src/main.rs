use crate::radio_frequency::RadioFrequency;
use shared::common::{
    constants::{
        C87B_FROM_SERVER_PORT, C87B_TO_SERVER_PORT, C88B_FROM_SERVER_PORT, C88B_TO_SERVER_PORT,
        GPS_FROM_SERVER_PORT, GPS_TO_SERVER_PORT, SATCOM_FROM_SERVER_PORT, SATCOM_TO_SERVER_PORT,
    },
    types::Channel,
};
use std::thread::park;

mod clients_registry;
mod radio_frequency;

#[tokio::main]
async fn main() {
    let freq_c87b: RadioFrequency =
        RadioFrequency::init(Channel::C87B, C87B_FROM_SERVER_PORT, C87B_TO_SERVER_PORT).await;
    let freq_c88b: RadioFrequency =
        RadioFrequency::init(Channel::C88B, C88B_FROM_SERVER_PORT, C88B_TO_SERVER_PORT).await;
    let freq_gps: RadioFrequency =
        RadioFrequency::init(Channel::GPS, GPS_FROM_SERVER_PORT, GPS_TO_SERVER_PORT).await;
    let freq_satcom: RadioFrequency = RadioFrequency::init(
        Channel::SATCOM,
        SATCOM_FROM_SERVER_PORT,
        SATCOM_TO_SERVER_PORT,
    )
    .await;

    freq_c87b.start();
    freq_c88b.start();
    freq_gps.start();
    freq_satcom.start();

    park();
}
