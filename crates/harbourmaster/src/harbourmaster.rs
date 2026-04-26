use std::sync::{Arc, Mutex};

use crate::{
    database_manager::{database_api::DatabaseApi, manager::DatabaseManager},
    fms::Fms,
    harbourmaster_ais::HarbourmasterAisRunner,
    harbourmaster_gps::HarbourmasterGps,
};

use shared::{
    antenna::Antenna,
    bitpacker::BitPacker,
    boats_registry::BoatsInfoRegistry,
    common::{
        constants::{
            C87B_EM_PORT, C87B_REC_PORT, C88B_EM_PORT, C88B_REC_PORT, GPS_EM_PORT, GPS_REC_PORT,
            SATCOM_EM_PORT, SATCOM_REC_PORT,
        },
        types::{AisPacket, Channel},
    },
    satcom::SatCom,
    satcom_message::SatComMessage,
};
use tokio::sync::{Semaphore, mpsc::channel};

pub struct Harbourmaster {
    ais: HarbourmasterAisRunner,
    gps: HarbourmasterGps,
    satcom: SatCom,
    fms: Fms,
    c87b_antenna: Antenna,
    c88b_antenna: Antenna,
    gps_antenna: Antenna,
    satcom_antenna: Antenna,
    database_api: DatabaseApi,
}

impl Harbourmaster {
    pub async fn init() -> Self {
        let (ais_tx, ais_rx) = channel::<AisPacket>(Semaphore::MAX_PERMITS);
        let (gps_tx, gps_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (sender_satcom_tx, sender_satcom_rx) = channel::<SatComMessage>(Semaphore::MAX_PERMITS);
        let (reader_satcom_tx, reader_satcom_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (fms_tx, fms_rx) = channel::<SatComMessage>(Semaphore::MAX_PERMITS);

        let (_, c87b_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (_, c88b_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (c_gps_tx, c_gps_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);
        let (c_satcom_tx, c_satcom_rx) = channel::<BitPacker>(Semaphore::MAX_PERMITS);

        let ant1: Antenna = Antenna::init(
            Some(ais_tx.clone()),
            None,
            None,
            c87b_rx,
            C87B_REC_PORT,
            C87B_EM_PORT,
            Channel::C87B,
        )
        .await;
        let ant2: Antenna = Antenna::init(
            Some(ais_tx),
            None,
            None,
            c88b_rx,
            C88B_REC_PORT,
            C88B_EM_PORT,
            Channel::C88B,
        )
        .await;
        let ant3: Antenna = Antenna::init(
            None,
            Some(gps_tx),
            None,
            c_gps_rx,
            GPS_REC_PORT,
            GPS_EM_PORT,
            Channel::GPS,
        )
        .await;
        let ant4: Antenna = Antenna::init(
            None,
            None,
            Some(reader_satcom_tx),
            c_satcom_rx,
            SATCOM_REC_PORT,
            SATCOM_EM_PORT,
            Channel::SATCOM,
        )
        .await;

        let boats_reg: Arc<BoatsInfoRegistry> = Arc::new(BoatsInfoRegistry::new());
        let db_manager: Arc<Mutex<DatabaseManager>> =
            Arc::new(Mutex::new(DatabaseManager::init().unwrap()));

        let ais: HarbourmasterAisRunner =
            HarbourmasterAisRunner::init(ais_rx, boats_reg.clone());
        let gps: HarbourmasterGps = HarbourmasterGps::init(gps_rx, c_gps_tx).await;
        let satcom: SatCom = SatCom::new(reader_satcom_rx, sender_satcom_rx, c_satcom_tx, fms_tx);
        let fms = Fms::new(
            boats_reg.clone(),
            db_manager.clone(),
            fms_rx,
            sender_satcom_tx,
        );

        let db_api: DatabaseApi = DatabaseApi::init(db_manager, boats_reg);

        Self {
            ais: ais,
            gps: gps,
            satcom: satcom,
            fms: fms,
            c87b_antenna: ant1,
            c88b_antenna: ant2,
            gps_antenna: ant3,
            satcom_antenna: ant4,
            database_api: db_api,
        }
    }

    pub async fn start(self) -> () {
        self.c87b_antenna.start().await;
        self.c88b_antenna.start().await;
        self.gps_antenna.start().await;
        self.satcom_antenna.start().await;
        self.ais.start().await;
        //self.gps.start().await;
        self.satcom.start().await;
        self.fms.start().await;
        self.database_api.start().await;
    }
}
