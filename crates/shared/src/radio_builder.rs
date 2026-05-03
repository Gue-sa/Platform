use crate::{
    antenna::Antenna,
    bitpacker::BitPacker,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::{
            C87B_FROM_SERVER_PORT, C87B_TO_SERVER_PORT, C88B_FROM_SERVER_PORT, C88B_TO_SERVER_PORT,
            GPS_FROM_SERVER_PORT, GPS_TO_SERVER_PORT, SATCOM_FROM_SERVER_PORT,
            SATCOM_TO_SERVER_PORT,
        },
        errors::RadioBuilderResult,
        types::{AisPacket, Channel, LogEvent},
    },
    satcom::SatCom,
    satcom_message::SatComMessage,
};
use std::sync::Arc;
use tokio::sync::{
    Semaphore,
    mpsc::{Receiver, Sender, channel},
};

pub async fn build_radio(
    logs_cli_tx: std::sync::mpsc::Sender<LogEvent>,
    mmsi: u32,
) -> RadioBuilderResult<(
    Receiver<AisPacket>,     //ais_rx
    Receiver<BitPacker>,     //gps_rx
    Receiver<SatComMessage>, //fms_rx or board_computer_rx
    Sender<BitPacker>,       //c87b_tx
    Sender<BitPacker>,       //c88b_tx
    Sender<BitPacker>,       //c_gps_tx
    Sender<SatComMessage>,   //sedner_satcom_tx
    Antenna,                 //c87b_antenna
    Antenna,                 //c88b_antenna
    Antenna,                 //gps_antenna
    Antenna,                 //satcom_antenna
    SatCom,                  //satcom
    Arc<BoatsInfoRegistry>,  //boats_reg
)> {
    let (ais_tx, ais_rx) = channel::<AisPacket>(Semaphore::MAX_PERMITS);
    let (gps_tx, gps_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
    let (sender_satcom_tx, sender_satcom_rx) = channel::<SatComMessage>(Semaphore::MAX_PERMITS);
    let (reader_satcom_tx, reader_satcom_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
    let (computer_tx, computer_rx) = channel::<SatComMessage>(Semaphore::MAX_PERMITS);

    let (c87b_tx, c87b_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
    let (c88b_tx, c88b_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
    let (c_gps_tx, c_gps_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
    let (c_satcom_tx, c_satcom_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);

    let ant1 = Antenna::init(
        Some(ais_tx.clone()),
        None,
        None,
        c87b_rx,
        C87B_TO_SERVER_PORT,
        C87B_FROM_SERVER_PORT,
        Channel::C87B,
    )
    .await?;
    let ant2 = Antenna::init(
        Some(ais_tx),
        None,
        None,
        c88b_rx,
        C88B_TO_SERVER_PORT,
        C88B_FROM_SERVER_PORT,
        Channel::C88B,
    )
    .await?;
    let ant3 = Antenna::init(
        None,
        Some(gps_tx),
        None,
        c_gps_rx,
        GPS_TO_SERVER_PORT,
        GPS_FROM_SERVER_PORT,
        Channel::GPS,
    )
    .await?;
    let ant4 = Antenna::init(
        None,
        None,
        Some(reader_satcom_tx),
        c_satcom_rx,
        SATCOM_TO_SERVER_PORT,
        SATCOM_FROM_SERVER_PORT,
        Channel::SatCom,
    )
    .await?;

    let satcom = SatCom::init(
        reader_satcom_rx,
        sender_satcom_rx,
        c_satcom_tx,
        computer_tx,
        logs_cli_tx,
        mmsi,
    );

    let boats_reg = Arc::new(BoatsInfoRegistry::new());

    Ok((
        ais_rx,
        gps_rx,
        computer_rx,
        c87b_tx,
        c88b_tx,
        c_gps_tx,
        sender_satcom_tx,
        ant1,
        ant2,
        ant3,
        ant4,
        satcom,
        boats_reg,
    ))
}
