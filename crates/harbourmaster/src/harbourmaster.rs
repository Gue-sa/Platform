use crate::{
    database_manager::{database_api::DatabaseApi, manager::DatabaseManager},
    fms::Fms,
    harbourmaster_ais::HarbourmasterAisRunner,
    harbourmaster_gps::HarbourmasterGps,
};
use shared::{
    antenna::Antenna, common::types::HarbourmasterResult, radio_builder::build_radio,
    satcom::SatCom,
};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

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
    pub async fn init() -> HarbourmasterResult<Self> {
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
        ) = build_radio().await?;

        let db_manager: Arc<Mutex<DatabaseManager>> =
            Arc::new(Mutex::new(DatabaseManager::init()?));
        let db_api: DatabaseApi = DatabaseApi::init(db_manager.clone(), boats_reg.clone());

        let ais: HarbourmasterAisRunner = HarbourmasterAisRunner::init(ais_rx, boats_reg.clone());
        let gps: HarbourmasterGps = HarbourmasterGps::init(gps_rx, c_gps_tx).await;
        let fms: Fms = Fms::init(boats_reg, db_manager, fms_rx, sender_satcom_tx);

        Ok(Self {
            ais: ais,
            gps: gps,
            satcom: satcom,
            fms: fms,
            c87b_antenna: ant1,
            c88b_antenna: ant2,
            gps_antenna: ant3,
            satcom_antenna: ant4,
            database_api: db_api,
        })
    }

    pub async fn start(self) -> HarbourmasterResult<()> {
        let _c87b_antenna_handle: JoinHandle<()> = self.c87b_antenna.start().await?;
        let _c88b_antenna_handle: JoinHandle<()> = self.c88b_antenna.start().await?;
        let _gps_antenna_handle: JoinHandle<()> = self.gps_antenna.start().await?;
        let _satcom_antenna_handle: JoinHandle<()> = self.satcom_antenna.start().await?;

        let _ais_handle: (JoinHandle<()>, JoinHandle<()>) = self.ais.start();
        //let _gps_handle: (JoinHandle<()>, JoinHandle<()>) = self.gps.start();
        let _satcom_handle: JoinHandle<()> = self.satcom.start();
        let _fms_handle: (JoinHandle<()>, JoinHandle<()>, JoinHandle<()>) = self.fms.start();

        self.database_api.start().await;

        Ok(())
    }
}
