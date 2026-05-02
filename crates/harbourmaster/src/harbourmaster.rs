use crate::{
    database_manager::{database_api::DatabaseApi, manager::DatabaseManager},
    fms::Fms,
    harbourmaster_ais::HarbourmasterAisRunner,
    harbourmaster_gps::HarbourmasterGps,
    harbourmaster_web_ui::{self, HarbourmasterWebUi},
};
use shared::{
    antenna::Antenna,
    common::{constants::HARBOURMASTER_MMSI, errors::HarbourmasterResult, types::LogEvent},
    config::Config,
    logs_cli::LogsCli,
    radio_builder::build_radio,
    satcom::SatCom,
};
use std::sync::{Arc, Mutex, mpsc::channel};
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
    web_ui: HarbourmasterWebUi,
    logs_cli: LogsCli,
}

impl Harbourmaster {
    pub async fn init() -> HarbourmasterResult<Self> {
        let config: Config = Config::load().unwrap();

        let (cli_tx, cli_rx) = channel::<LogEvent>();
        let cli: LogsCli = LogsCli::new(
            cli_rx,
            (*config.harbourmaster_sys_logs_filename().clone()).to_string(),
            (*config.harbourmaster_ais_logs_filename().clone()).to_string(),
            (*config.harbourmaster_gps_logs_filename().clone()).to_string(),
            (*config.harbourmaster_satcom_logs_filename().clone()).to_string(),
            (*config.harbourmaster_computer_logs_filename().clone()).to_string(),
            "Logs Armateur (interface web : localhost:8080)",
        );

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
        ) = build_radio(cli_tx.clone(), HARBOURMASTER_MMSI).await?;

        let db_manager: Arc<Mutex<DatabaseManager>> =
            Arc::new(Mutex::new(DatabaseManager::init()?));
        let db_api: DatabaseApi =
            DatabaseApi::init(db_manager.clone(), boats_reg.clone(), cli_tx.clone());

        let ais: HarbourmasterAisRunner =
            HarbourmasterAisRunner::init(ais_rx, boats_reg.clone(), cli_tx.clone());
        let gps: HarbourmasterGps = HarbourmasterGps::init(gps_rx, c_gps_tx, cli_tx.clone()).await;
        let fms: Fms = Fms::init(
            boats_reg,
            db_manager,
            fms_rx,
            sender_satcom_tx,
            cli_tx.clone(),
        );

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
            web_ui: HarbourmasterWebUi::new().await,
            logs_cli: cli,
        })
    }

    pub async fn start(self) -> HarbourmasterResult<()> {
        let config: Config = Config::load().unwrap();

        let _c87b_antenna_handle: JoinHandle<()> = self.c87b_antenna.start().await?;
        let _c88b_antenna_handle: JoinHandle<()> = self.c88b_antenna.start().await?;
        let _gps_antenna_handle: JoinHandle<()> = self.gps_antenna.start().await?;
        let _satcom_antenna_handle: JoinHandle<()> = self.satcom_antenna.start().await?;

        let _ais_handle: (JoinHandle<()>, JoinHandle<()>) = self.ais.start();
        let _gps_handle: (JoinHandle<()>, JoinHandle<()>) = self.gps.start();
        let _satcom_handle: JoinHandle<()> = self.satcom.start();
        let _fms_handle: (JoinHandle<()>, JoinHandle<()>, JoinHandle<()>) = self.fms.start();

        let _database_api_handle: Option<JoinHandle<()>> = if *config.api() {
            Some(self.database_api.start().await)
        } else {
            None
        };

        let _harbourmaster_wui_handle: Option<JoinHandle<()>> = if *config.wui() {
            Some(self.web_ui.start().await)
        } else {
            None
        };

        let _logs_cli_handle: Option<JoinHandle<()>> = if *config.cli() {
            Some(self.logs_cli.run().unwrap())
        } else {
            None
        };

        std::thread::park();

        Ok(())
    }
}
