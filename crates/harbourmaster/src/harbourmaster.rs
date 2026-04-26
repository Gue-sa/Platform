use crate::{
    database_manager::{database_api::DatabaseApi, manager::DatabaseManager},
    fms::Fms,
    harbourmaster_ais::HarbourmasterAisRunner,
    harbourmaster_gps::HarbourmasterGps,
};
use shared::{antenna::Antenna, radio_builder::build_radio, satcom::SatCom};
use std::sync::{Arc, Mutex};

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
        let (
            ais_rx,
            gps_rx,
            fms_rx,
            _,
            _,
            c_gps_tx,
            sender_satcom_tx,
            ant1,
            ant2,
            ant3,
            ant4,
            satcom,
            boats_reg,
        ) = build_radio().await;

        let db_manager: Arc<Mutex<DatabaseManager>> =
            Arc::new(Mutex::new(DatabaseManager::init().unwrap()));
        let db_api: DatabaseApi = DatabaseApi::init(db_manager.clone(), boats_reg.clone());

        let ais: HarbourmasterAisRunner = HarbourmasterAisRunner::init(ais_rx, boats_reg.clone());
        let gps: HarbourmasterGps = HarbourmasterGps::init(gps_rx, c_gps_tx).await;
        let fms: Fms = Fms::new(boats_reg, db_manager, fms_rx, sender_satcom_tx);

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
